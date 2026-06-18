<script module lang="ts">
  import DOMPurify from 'dompurify';

  // メール本文の sanitize（DOMPurify）。script/style/iframe 等の危険タグ・イベントハンドラ・
  // 危険スキームを除去し、http(s) リンクのみ data-href 化する（sandbox iframe の自己ナビゲーションを
  // 防ぎ、クリックは bridge 経由で外部ブラウザに開く）。それ以外の href は除去。
  // フックは初回 sanitize 時に一度だけ登録（モジュールスコープ）。ビルド時(window 無し)の
  // 実行を避けるため、addHook は import 時ではなく遅延登録する。
  let _hookReady = false;
  function ensureHook() {
    if (_hookReady) return;
    _hookReady = true;
    DOMPurify.addHook('afterSanitizeAttributes', (node) => {
      const el = node as Element;
      if (el.tagName === 'A') {
        const href = el.getAttribute('href') ?? '';
        el.removeAttribute('href');
        if (/^https?:/i.test(href)) el.setAttribute('data-href', href);
      }
    });
  }

  function sanitizeHtml(html: string): string {
    ensureHook();
    return DOMPurify.sanitize(html, {
      FORBID_TAGS: ['script', 'style', 'iframe', 'object', 'embed', 'form', 'base', 'meta', 'link', 'applet'],
      FORBID_ATTR: ['data-href'],
      WHOLE_DOCUMENT: false
    }) as string;
  }
</script>

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { slide } from 'svelte/transition';
  import Button from './Button.svelte';
  import AiPanel from './AiPanel.svelte';
  import CalendarPanel from './CalendarPanel.svelte';
  import EventCard from './EventCard.svelte';
  import type { MailDetail, Attachment, CalendarEvent } from '$lib/types';
  import { formatMailDate } from '$lib/store';

  let { mail, onArchive, onDelete, onStar, onReply, onForward, onUseAiReply, onDownloadAttachment, onFetchAttachmentData, llmConfig, smtpConfig, calendarName = '仕事', calendarNames = [], calendarProvider = 'apple', dateFormat = 'YYYY/MM/DD HH:mm:ss', timezone = 'Asia/Tokyo', imageLoadingPolicy = 'allow', imageWhitelist = [] }: {
    mail: MailDetail | null;
    onArchive: () => void;
    onDelete: () => void;
    onStar: (add: boolean) => void;
    onReply: () => void;
    onForward: () => void;
    onUseAiReply: (text: string) => void;
    onDownloadAttachment: (partIndex: number, filename: string) => void;
    onFetchAttachmentData: (partIndex: number) => Promise<string>;
    llmConfig: { base_url: string; model: string; api_key: string };
    smtpConfig?: { email: string; auth_type: string; password: string; access_token: string; smtp_host: string; smtp_port: number } | null;
    calendarName?: string;
    calendarNames?: string[];
    calendarProvider?: string;
    dateFormat?: string;
    timezone?: string;
    imageLoadingPolicy?: 'block' | 'whitelist' | 'allow';
    imageWhitelist?: string[];
  } = $props();

  // リモート画像の表示可否。policy=block は常に不可、allow は常に可、
  // whitelist は送信者(From)がホワイトリスト(メール or ドメイン)に一致する場合のみ可。
  // 実アドレスは最後の <...> を採用（表示名に埋め込んだ偽 <...> に騙されないため）。
  // 注意: From は DKIM/SPF 未検証で詐称可能。本ホワイトリストはセキュリティ境界では
  // なく「すべて許可」より厳しくする利便性目的（非一致は fail-safe でブロック）。
  function senderAllowed(from: string, list: string[]): boolean {
    if (!from || !list || list.length === 0) return false;
    const angles = from.match(/<([^>]+)>/g);
    const addr = (angles && angles.length ? angles[angles.length - 1].slice(1, -1) : from).trim().toLowerCase();
    if (!addr) return false;
    const domain = addr.includes('@') ? addr.slice(addr.indexOf('@') + 1) : '';
    return list.some((e) => {
      const entry = e.trim().toLowerCase().replace(/^@/, '');
      if (!entry) return false;
      return addr === entry || domain === entry || (domain !== '' && domain.endsWith('.' + entry));
    });
  }
  const imagesAllowed = $derived(
    imageLoadingPolicy === 'allow' ? true :
    imageLoadingPolicy === 'block' ? false :
    senderAllowed(mail?.from ?? '', imageWhitelist)
  );

  let openPanels: Set<string> = $state(new Set());
  let starred = $state(false);
  let previewAtt: Attachment | null = $state(null);
  let previewData: string | null = $state(null);
  let previewLoading = $state(false);
  let prevUid: number | null = $state(null);
  let translatedBody: string | null = $state(null);
  let translating = $state(false);
  let icsEvents: CalendarEvent[] = $state([]);
  let conflictsMap: Record<string, string[]> = $state({});
  let iframeEl: HTMLIFrameElement | undefined = $state(undefined);
  let iframeHeight = $state(200);
  let detailEl: HTMLDivElement | undefined = $state(undefined);
  let compact = $state(false);

  // Reset panels when mail changes
  $effect(() => { if (mail) starred = (mail as any).starred ?? false; });

  $effect(() => {
    if (!detailEl) return;
    const ro = new ResizeObserver(([e]) => { compact = e.contentRect.width <= 600; });
    ro.observe(detailEl);
    return () => ro.disconnect();
  });

  $effect(() => {
    const uid = mail?.uid ?? null;
    if (uid !== prevUid) {
      openPanels = new Set();
      translatedBody = null;
      icsEvents = [];
      conflictsMap = {};
      prevUid = uid;
      // Detect and parse ics attachment
      const icsAtt = mail?.attachments.find(a => a.filename.endsWith('.ics'));
      if (icsAtt) {
        const currentUid = uid;
        onFetchAttachmentData(icsAtt.index).then(b64 =>
          invoke<CalendarEvent[]>('parse_ics_attachment', { data: b64 })
        ).then(async evts => {
          if (mail?.uid !== currentUid) return;
          if (evts.length > 0) {
            icsEvents = evts;
            if (smtpConfig?.auth_type === 'oauth') {
              const toRfc = (s: string) => {
                // Already RFC3339 format
                if (/^\d{4}-\d{2}-\d{2}T/.test(s)) return s.endsWith('Z') || /[+-]\d{2}:\d{2}$/.test(s) ? s : s + '+09:00';
                // ICS format: 20260612T193000Z or 20260612T193000
                const isUtc = s.endsWith('Z');
                const digits = s.replace(/[^0-9T]/g, '');
                const m = digits.match(/^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})?/);
                if (!m) return s;
                const base = `${m[1]}-${m[2]}-${m[3]}T${m[4]}:${m[5]}:${m[6]||'00'}`;
                return isUtc ? base + 'Z' : base + '+09:00';
              };
              for (let i = 0; i < icsEvents.length; i++) {
                const ev = icsEvents[i];
                if (!ev.uid) continue;
                try {
                  const status = await invoke<string>('get_calendar_event_status', {
                    accessToken: smtpConfig.access_token, icsUid: ev.uid, myEmail: smtpConfig.email
                  });
                  if (status !== 'unknown') icsEvents[i] = { ...ev, status: status.toUpperCase() };
                } catch {}
                try {
                  const c = await invoke<string[]>('check_calendar_conflicts', {
                    accessToken: smtpConfig.access_token, timeMin: toRfc(ev.dtstart), timeMax: toRfc(ev.dtend), excludeUid: ev.uid
                  });
                  if (c.length > 0) conflictsMap[ev.uid] = c;
                } catch {}
              }
              icsEvents = [...icsEvents]; // trigger reactivity
            }
          }
        }).catch(() => {});
      }
    }
  });

  // sandbox iframe（allow-scripts / same-origin なし）内の注入スクリプトから
  // postMessage を受け取り、リンクは外部ブラウザで開き、本文高さを反映する。
  // 親から contentDocument に click リスナーを張る方式は、scripting 無効の
  // sandbox document では WebKit/WKWebView で発火しないため使えない。
  // セキュリティ: 送信元(contentWindow)一致と http/https スキームを検証してから open。
  $effect(() => {
    function onMessage(e: MessageEvent) {
      if (!iframeEl || e.source !== iframeEl.contentWindow) return;
      const d = e.data as { t?: string; h?: number; u?: string } | null;
      if (!d || typeof d !== 'object') return;
      if (d.t === 'smartam-h' && typeof d.h === 'number') {
        iframeHeight = Math.max(40, Math.min(d.h, 20000));
      } else if (d.t === 'smartam-link' && typeof d.u === 'string') {
        if (d.u.startsWith('http://') || d.u.startsWith('https://')) {
          invoke('open_external_url', { url: d.u }).catch(() => {});
        }
      }
    }
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  });

  // sanitizeHtml は上部の <script module>（DOMPurify ベース）で定義。buildSrcdoc から利用する。

  // 高さ計測 + リンク仲介の信頼スクリプト。same-origin を外した sandbox
  // (allow-scripts のみ) 内で click と本文高さを親へ postMessage する。
  // ⚠️ 本番ビルドでは srcdoc が Tauri の CSP を継承し、Tauri が script-src に nonce を
  //   注入して 'unsafe-inline' が無効化される。よってこのスクリプトの SHA-256 を
  //   tauri.conf.json の script-src に 'sha256-...' として登録し実行を許可している。
  //   この文字列を変更したら必ず再計算: SHA-256(MAIL_BRIDGE_JS, utf8) を base64 化し
  //   'sha256-' を前置して tauri.conf.json を更新すること。
  const MAIL_BRIDGE_JS = `(function(){var PO=(document.querySelector('meta[name=smartam-po]')||{}).content||'*';function s(m){parent.postMessage(m,PO);}function h(){s({t:'smartam-h',h:document.body.scrollHeight+16});}document.addEventListener('click',function(e){var t=e.target,a=t&&t.closest?t.closest('a[data-href]'):null;if(a){e.preventDefault();var u=a.getAttribute('data-href');if(u)s({t:'smartam-link',u:u});}});window.addEventListener('load',h);if(document.readyState!=='loading')h();try{new ResizeObserver(h).observe(document.body);}catch(_){}setTimeout(h,120);setTimeout(h,600);})();`;

  function buildSrcdoc(html: string): string {
    const safe = sanitizeHtml(html);
    const nonce = crypto.randomUUID().replace(/-/g, '');
    const bridge = MAIL_BRIDGE_JS;
    // bridge の postMessage 送信先(targetOrigin)を親オリジンに限定するため meta で注入。
    // 親オリジンは可変(dev: http://localhost:5173 / 本番: tauri オリジン)なので
    // script 本体(=ハッシュ対象)には含めず meta 経由で渡す（ハッシュ安定維持）。
    const po = (typeof window !== 'undefined' && window.location?.origin ? window.location.origin : '').replace(/&/g, '&amp;').replace(/"/g, '&quot;');
    const imgSrc = imagesAllowed ? 'img-src data: https:' : 'img-src data:';
    return `<html><head><meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'nonce-${nonce}'; style-src 'unsafe-inline'; ${imgSrc}"><meta name="smartam-po" content="${po}"><style>
body{margin:0;padding:16px;font:13px/1.7 -apple-system,system-ui,'Segoe UI',Roboto,sans-serif;color:#1a1a1a;background:#fff;word-break:break-word;overflow-x:hidden}
img{max-width:100%;height:auto}
table{border-collapse:collapse;max-width:100%;width:100%}
td,th{padding:4px 8px;word-break:break-word}
a{color:#1e66f5;cursor:pointer}
blockquote{border-left:3px solid #ddd;padding-left:12px;margin:8px 0;color:#666}
*{max-width:100%;box-sizing:border-box}
@media (prefers-color-scheme: dark) {
  body { background: #1e1e2e; color: #cdd6f4 }
  a { color: #89b4fa }
  blockquote { border-left-color: #585b70; color: #a6adc8 }
}
</style></head><body>${safe}<script nonce="${nonce}">${bridge}</scr` + `ipt></body></html>`;
  }

  function linkifyText(text: string): string {
    const escaped = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    return escaped.replace(/(https?:\/\/[^\s<>&"]+)/g, '<a href="$1" rel="noopener noreferrer">$1</a>');
  }

  function togglePanel(type: string) {
    const next = new Set(openPanels);
    if (next.has(type)) next.delete(type); else next.add(type);
    openPanels = next;
  }
  function getMailText(): string {
    if (!mail) return '';
    if (mail.body_text.trim()) return mail.body_text;
    // Fallback: strip HTML tags from body_html
    if (typeof DOMParser === 'undefined') return mail.body_html.replace(/<[^>]*>/g, '');
    const doc = new DOMParser().parseFromString(mail.body_html, 'text/html');
    return doc.body.textContent || '';
  }
  async function translateInline() {
    if (translatedBody !== null) { translatedBody = null; return; }
    translating = true;
    try {
      const source = mail?.body_html || getMailText();
      translatedBody = await invoke<string>('ai_translate', { llm: llmConfig, text: source, targetLang: '日本語' });
    } catch { translatedBody = null; }
    finally { translating = false; }
  }

  async function openPreview(att: Attachment) {
    previewAtt = att;
    previewData = null;
    previewLoading = true;
    try {
      previewData = await onFetchAttachmentData(att.index);
    } catch { previewData = null; }
    finally { previewLoading = false; }
  }

  function previewType(mime: string, name: string): 'image' | 'pdf' | 'video' | 'text' | 'md' | 'unknown' {
    if (mime.startsWith('image/')) return 'image';
    if (mime === 'application/pdf') return 'pdf';
    if (mime.startsWith('video/')) return 'video';
    if (name.endsWith('.md') || name.endsWith('.markdown')) return 'md';
    if (mime.startsWith('text/') || /\.(txt|csv|json|xml|html|css|js|ts|py|rs|sh|yml|yaml|toml|log|ini|cfg)$/i.test(name)) return 'text';
    return 'unknown';
  }

  function dataUrl(mime: string, b64: string) { return `data:${mime};base64,${b64}`; }

  function decodeText(b64: string): string {
    try { return new TextDecoder().decode(Uint8Array.from(atob(b64), c => c.charCodeAt(0))); }
    catch { return atob(b64); }
  }

  function renderMd(text: string): string {
    const escaped = text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return escaped
      .replace(/^### (.+)$/gm, '<h3>$1</h3>')
      .replace(/^## (.+)$/gm, '<h2>$1</h2>')
      .replace(/^# (.+)$/gm, '<h1>$1</h1>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.+?)\*/g, '<em>$1</em>')
      .replace(/`(.+?)`/g, '<code>$1</code>')
      .replace(/\n/g, '<br>');
  }

  function formatSize(bytes: number) {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / 1048576).toFixed(1)}MB`;
  }
</script>

<div class="detail" class:compact bind:this={detailEl}>
  {#if mail}
    <div class="header">
      <h2 class="subject">{mail.subject}</h2>
      <div class="meta">From: {mail.from}  |  {formatMailDate(mail.date, dateFormat, timezone)}</div>
      <div class="meta">To: {mail.to}</div>
      {#if mail.cc}<div class="meta">Cc: {mail.cc}</div>{/if}
      {#if mail.bcc}<div class="meta">Bcc: {mail.bcc}</div>{/if}
    </div>
    <div class="actions">
      <Button icon="reply" label="返信" title="返信" onclick={onReply} />
      <Button icon="forward" label="転送" title="転送" onclick={onForward} />
      <span class="divider"></span>
      <Button icon="archive" label="アーカイブ" title="アーカイブ" onclick={onArchive} />
      <Button icon="trash" label="削除" variant="danger" title="削除" onclick={onDelete} />
      <Button icon="star" label="スター" variant={starred ? 'starred' : ''} title="スター" onclick={() => { starred = !starred; onStar(starred); }} />
      <span class="divider"></span>
      <Button icon="summary" label="要約" variant="ai-summary" active={openPanels.has('summary')} title="AI要約" onclick={() => togglePanel('summary')} kbd="y" />
      <Button icon="draft" label="下書き" variant="ai-draft" active={openPanels.has('nuance')} title="返信下書き" onclick={() => togglePanel('nuance')} kbd="d" />
      <Button icon="translate" label="翻訳" variant="ai-translate" active={translatedBody !== null || translating} title="翻訳" onclick={translateInline} disabled={translating} kbd="t" />
      <Button icon="calendar" label="カレンダー" variant="ai-calendar" active={openPanels.has('calendar')} title="カレンダー登録" onclick={() => togglePanel('calendar')} kbd="l" />
    </div>

    {#if mail.attachments.length > 0}
      <div class="attachments">
        {#each [...mail.attachments].sort((a, b) => (b.filename.endsWith('.ics') ? 1 : 0) - (a.filename.endsWith('.ics') ? 1 : 0)) as att}
          <button class="att-chip" onclick={() => openPreview(att)}>
            📄 {att.filename} ({formatSize(att.size)})
          </button>
        {/each}
      </div>
    {/if}

    {#if openPanels.has('summary')}
      <div transition:slide={{ duration: 150 }}>
        <AiPanel mailBody={getMailText()} {llmConfig} onClose={() => togglePanel('summary')} onUseReply={(text) => { togglePanel('nuance'); onUseAiReply(text); }} initialPanel="summary" />
      </div>
    {/if}
    {#if openPanels.has('nuance')}
      <div transition:slide={{ duration: 150 }}>
        <AiPanel mailBody={getMailText()} {llmConfig} onClose={() => togglePanel('nuance')} onUseReply={(text) => { togglePanel('nuance'); onUseAiReply(text); }} initialPanel="nuance" />
      </div>
    {/if}
    {#if openPanels.has('calendar')}
      <div transition:slide={{ duration: 150 }}>
        <CalendarPanel mailBody={getMailText()} {llmConfig} {calendarName} {calendarNames} provider={calendarProvider} accessToken={smtpConfig?.access_token ?? ''} onClose={() => togglePanel('calendar')} />
      </div>
    {/if}

    {#each icsEvents as ev, i}
      <EventCard event={ev} conflicts={conflictsMap[ev.uid] ?? []}
        onAccept={async () => {
          if (smtpConfig?.auth_type === 'oauth') {
            await invoke('respond_google_calendar_invite', { accessToken: smtpConfig.access_token, icsUid: ev.uid, myEmail: smtpConfig.email, accept: true });
          } else {
            await invoke('respond_calendar_invite', { smtp: smtpConfig, event: ev, accept: true });
          }
          icsEvents[i] = { ...ev, status: 'ACCEPTED' };
        }}
        onDecline={async () => {
          if (smtpConfig?.auth_type === 'oauth') {
            await invoke('respond_google_calendar_invite', { accessToken: smtpConfig.access_token, icsUid: ev.uid, myEmail: smtpConfig.email, accept: false });
          } else {
            await invoke('respond_calendar_invite', { smtp: smtpConfig, event: ev, accept: false });
          }
          icsEvents[i] = { ...ev, status: 'DECLINED' };
        }}
      />
    {/each}

    {#if translatedBody !== null}
      {#if mail.body_html}
        <iframe class="mail-iframe translated" title="メール本文(翻訳)" srcdoc={buildSrcdoc(translatedBody)} sandbox="allow-scripts" style:height="{iframeHeight}px" bind:this={iframeEl}></iframe>
      {:else}
        <div class="body translated">{translatedBody}</div>
      {/if}
    {:else if mail.body_html}
      <iframe class="mail-iframe" title="メール本文" srcdoc={buildSrcdoc(mail.body_html)} sandbox="allow-scripts" style:height="{iframeHeight}px" bind:this={iframeEl}></iframe>
    {:else if mail.body_text.trim()}
      <div class="body">{@html linkifyText(mail.body_text)}</div>
    {/if}
  {:else}
    <div class="empty">メールを選択してください</div>
  {/if}
</div>

{#if previewAtt}
  {@const pt = previewType(previewAtt.mime_type, previewAtt.filename)}
  <div class="pov" onclick={() => { previewAtt = null; previewData = null; }}>
    <div class="pom" onclick={(e) => e.stopPropagation()}>
      <div class="poh">📄 {previewAtt.filename}<button class="x" onclick={() => { previewAtt = null; previewData = null; }}>✕</button></div>
      <div class="poa">
        {#if previewLoading}
          <div class="poa-msg">⏳ 読み込み中...</div>
        {:else if !previewData}
          <div class="poa-msg">プレビューを読み込めませんでした</div>
        {:else if pt === 'image'}
          <img src={dataUrl(previewAtt.mime_type, previewData)} alt={previewAtt.filename} class="poa-img" />
        {:else if pt === 'pdf'}
          <iframe src={dataUrl('application/pdf', previewData)} class="poa-frame" title="PDF"></iframe>
        {:else if pt === 'video'}
          <video src={dataUrl(previewAtt.mime_type, previewData)} controls class="poa-video"></video>
        {:else if pt === 'md'}
          <div class="poa-text md">{@html renderMd(decodeText(previewData))}</div>
        {:else if pt === 'text'}
          <pre class="poa-text">{decodeText(previewData)}</pre>
        {:else}
          <div class="poa-msg">このファイル形式のプレビューには対応していません</div>
        {/if}
      </div>
      <div class="pof">
        <span class="fi">{formatSize(previewAtt.size)} | {previewAtt.mime_type}</span>
        <button class="bd" onclick={() => { previewAtt = null; previewData = null; }}>閉じる</button>
        <button class="bd primary" onclick={() => { onDownloadAttachment(previewAtt!.index, previewAtt!.filename); previewAtt = null; previewData = null; }}>⬇ ダウンロード</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .detail { flex:1;overflow-y:auto;padding:16px }
  .subject { font-size:16px;font-weight:700;margin-bottom:4px }
  .meta { color:var(--overlay);font-size:10px;margin-bottom:2px }
  .actions { display:flex;gap:4px;margin:12px 0;flex-wrap:wrap;align-items:center }
  .divider { width:1px;height:24px;background:var(--line, var(--surface1));margin:0 4px }
  .attachments { display:flex;gap:6px;margin:8px 0;flex-wrap:wrap }
  .att-chip { padding:4px 10px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .att-chip:hover { border-color:var(--mauve) }
  .body { font-size:13px;line-height:1.7;white-space:pre-wrap;margin-top:12px;background:var(--paper-wh, #fff);color:var(--ink, #1a1a1a);padding:16px;border-radius:8px }
  .body :global(a) { color:#1e66f5 }
  .mail-iframe { width:100%;border:none;margin-top:12px;border-radius:8px;background:#fff }
  .mail-iframe.translated { border:1px solid var(--blue) }
  .translated { border:1px solid var(--blue);background:var(--mantle);color:var(--text) }
  .empty { color:var(--overlay);text-align:center;padding:80px 0;font-size:14px }
  .pov { position:fixed;inset:0;background:rgba(0,0,0,.5);display:flex;align-items:center;justify-content:center;z-index:55 }
  .pom { width:820px;height:600px;background:var(--base);border:1px solid var(--surface1);border-radius:8px;display:flex;flex-direction:column }
  .poh { display:flex;justify-content:space-between;padding:10px 16px;font-weight:700;font-size:13px;border-bottom:1px solid var(--surface1) }
  .x { background:none;border:none;color:var(--overlay);cursor:pointer;font-size:16px }
  .poa { flex:1;display:flex;align-items:center;justify-content:center;background:var(--mantle);margin:12px;border-radius:6px;overflow:auto }
  .poa-msg { color:var(--overlay);font-size:14px }
  .poa-img { max-width:100%;max-height:100%;object-fit:contain }
  .poa-frame { width:100%;height:100%;border:none;border-radius:6px }
  .poa-video { max-width:100%;max-height:100% }
  .poa-text { width:100%;height:100%;padding:16px;font-size:12px;line-height:1.6;color:var(--text);overflow:auto;white-space:pre-wrap;word-break:break-word;margin:0;font-family:inherit }
  .poa-text.md { font-family:inherit;white-space:normal }
  .poa-text.md :global(h1) { font-size:18px;font-weight:700;margin:8px 0 }
  .poa-text.md :global(h2) { font-size:15px;font-weight:700;margin:6px 0 }
  .poa-text.md :global(h3) { font-size:13px;font-weight:700;margin:4px 0 }
  .poa-text.md :global(code) { background:var(--surface0);padding:1px 4px;border-radius:3px;font-family:monospace;font-size:11px }
  .pof { display:flex;align-items:center;padding:10px 16px;gap:8px;border-top:1px solid var(--surface1) }
  .fi { color:var(--overlay);font-size:10px;flex:1 }
  .bd { padding:6px 16px;border-radius:6px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);cursor:pointer;font-size:11px }
  .bd.primary { background:var(--mauve);color:var(--base);border:none;font-weight:700 }
  .compact :global(.btn .label) { display:none }
  .compact :global(.btn .kbd) { display:none }
  .compact :global(.btn) { width:34px;padding:8px;justify-content:center }
</style>
