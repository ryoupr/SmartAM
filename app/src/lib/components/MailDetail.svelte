<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { slide } from 'svelte/transition';
  import AiPanel from './AiPanel.svelte';
  import CalendarPanel from './CalendarPanel.svelte';
  import EventCard from './EventCard.svelte';
  import type { MailDetail, Attachment, CalendarEvent } from '$lib/types';

  let { mail, onArchive, onDelete, onStar, onReply, onForward, onUseAiReply, onDownloadAttachment, onFetchAttachmentData, llmConfig, smtpConfig, calendarName = '仕事', calendarNames = [] }: {
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
  } = $props();

  let openPanels: Set<string> = $state(new Set());
  let starred = $state(false);
  let previewAtt: Attachment | null = $state(null);
  let previewData: string | null = $state(null);
  let previewLoading = $state(false);
  let prevUid: number | null = $state(null);
  let translatedBody: string | null = $state(null);
  let translating = $state(false);
  let icsEvent: CalendarEvent | null = $state(null);

  // Reset panels when mail changes
  $effect(() => {
    const uid = mail?.uid ?? null;
    if (uid !== prevUid) {
      openPanels = new Set();
      translatedBody = null;
      icsEvent = null;
      prevUid = uid;
      // Detect and parse ics attachment
      const icsAtt = mail?.attachments.find(a => a.filename.endsWith('.ics'));
      if (icsAtt) {
        onFetchAttachmentData(icsAtt.index).then(b64 =>
          invoke<CalendarEvent[]>('parse_ics_attachment', { data: b64 })
        ).then(evts => { if (evts.length > 0) icsEvent = evts[0]; }).catch(() => {});
      }
    }
  });

  function sanitizeHtml(html: string): string {
    return html
      .replace(/<script[\s\S]*?<\/script>/gi, '')
      .replace(/<style[\s\S]*?<\/style>/gi, '')
      .replace(/\son\w+\s*=\s*"[^"]*"/gi, '')
      .replace(/\son\w+\s*=\s*'[^']*'/gi, '')
      .replace(/href\s*=\s*"javascript:[^"]*"/gi, 'href="#"')
      .replace(/<(iframe|object|embed|form)[^>]*>[\s\S]*?<\/\1>/gi, '')
      .replace(/<(iframe|object|embed|form)[^>]*\/?>/gi, '');
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
    const div = document.createElement('div');
    div.innerHTML = mail.body_html;
    return div.textContent || div.innerText || '';
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
    return text
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

<div class="detail">
  {#if mail}
    <div class="header">
      <h2 class="subject">{mail.subject}</h2>
      <div class="meta">From: {mail.from}  |  {mail.date}</div>
      <div class="meta">To: {mail.to}</div>
    </div>
    <div class="actions">
      <button class="btn" title="返信" onclick={onReply}>↩</button>
      <button class="btn" title="転送" onclick={onForward}>→</button>
      <button class="btn" title="アーカイブ" onclick={onArchive}>📦</button>
      <button class="btn" title="削除" onclick={onDelete}>🗑</button>
      <button class="btn" title="スター" class:starred onclick={() => { starred = !starred; onStar(starred); }}>⭐</button>
      <button class="btn ai" title="カレンダー登録" style="--ac:var(--blue)" class:active={openPanels.has('calendar')} onclick={() => togglePanel('calendar')}>📅</button>
      <button class="btn ai" title="AI要約" style="--ac:var(--green)" class:active={openPanels.has('summary')} onclick={() => togglePanel('summary')}>📝</button>
      <button class="btn ai" title="返信下書き" style="--ac:var(--yellow)" class:active={openPanels.has('nuance')} onclick={() => togglePanel('nuance')}>✍</button>
      <button class="btn ai" title="翻訳" style="--ac:var(--blue)" class:active={translatedBody !== null || translating} onclick={translateInline} disabled={translating}>{translating ? '⏳' : '🌐'}</button>
    </div>

    {#if mail.attachments.length > 0}
      <div class="attachments">
        {#each mail.attachments as att}
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
        <CalendarPanel mailBody={getMailText()} {llmConfig} {calendarName} {calendarNames} onClose={() => togglePanel('calendar')} />
      </div>
    {/if}

    {#if icsEvent}
      <EventCard event={icsEvent}
        onAccept={async () => {
          if (smtpConfig?.auth_type === 'oauth') {
            await invoke('respond_google_calendar_invite', { accessToken: smtpConfig.access_token, icsUid: icsEvent!.uid, myEmail: smtpConfig.email, accept: true });
          } else {
            await invoke('respond_calendar_invite', { smtp: smtpConfig, event: icsEvent, accept: true });
          }
        }}
        onDecline={async () => {
          if (smtpConfig?.auth_type === 'oauth') {
            await invoke('respond_google_calendar_invite', { accessToken: smtpConfig.access_token, icsUid: icsEvent!.uid, myEmail: smtpConfig.email, accept: false });
          } else {
            await invoke('respond_calendar_invite', { smtp: smtpConfig, event: icsEvent, accept: false });
          }
        }}
      />
    {/if}

    {#if translatedBody !== null}
      {#if mail.body_html}
        <div class="body body-html translated">{@html sanitizeHtml(translatedBody)}</div>
      {:else}
        <div class="body translated">{translatedBody}</div>
      {/if}
    {:else if mail.body_html}
      <div class="body body-html">{@html sanitizeHtml(mail.body_html)}</div>
    {:else}
      <div class="body">{mail.body_text}</div>
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
  .actions { display:flex;gap:4px;margin:12px 0;flex-wrap:wrap }
  .btn { padding:4px 8px;border-radius:6px;border:1px solid var(--surface1);background:var(--surface0);cursor:pointer;font-size:14px }
  .btn:hover { border-color:var(--subtext) }
  .btn.ai { border-color:var(--ac) }
  .btn.ai.active { background:var(--ac) }
  .btn.starred { background:var(--yellow) }
  .attachments { display:flex;gap:6px;margin:8px 0;flex-wrap:wrap }
  .att-chip { padding:4px 10px;border-radius:4px;border:1px solid var(--surface1);background:var(--surface0);color:var(--text);font-size:10px;cursor:pointer }
  .att-chip:hover { border-color:var(--mauve) }
  .body { font-size:13px;line-height:1.7;white-space:pre-wrap;margin-top:12px;background:#fff;color:#1a1a1a;padding:16px;border-radius:8px }
  .body :global(a) { color:#1e66f5 }
  .body-html { white-space:normal;word-break:break-word;overflow-x:auto;max-width:100% }
  .body-html :global(*) { max-width:100%!important;box-sizing:border-box;color:inherit }
  .body-html :global(body), .body-html :global(html) { margin:0;padding:0;width:100%!important }
  .body-html :global(a) { color:#1e66f5!important;text-decoration:underline }
  .body-html :global(img) { max-width:100%!important;height:auto!important }
  .body-html :global(table) { border-collapse:collapse;max-width:100%!important;width:auto!important;table-layout:fixed }
  .body-html :global(td), .body-html :global(th) { padding:4px 8px;word-break:break-word }
  .body-html :global(blockquote) { border-left:3px solid #ddd;padding-left:12px;margin:8px 0;color:#666 }
  .translated { border:1px solid var(--blue);background:var(--mantle);color:var(--text) }
  .body-html :global(div), .body-html :global(p), .body-html :global(span) { max-width:100%!important }
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
</style>
