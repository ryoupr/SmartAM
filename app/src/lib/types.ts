export interface MailSummary {
  uid: number;
  from: string;
  subject: string;
  date: string;
  seen: boolean;
}

export interface Attachment {
  index: number;
  filename: string;
  mime_type: string;
  size: number;
}

export interface MailDetail {
  uid: number;
  from: string;
  to: string;
  subject: string;
  date: string;
  body_text: string;
  body_html: string;
  attachments: Attachment[];
}

export interface AccountConfig {
  email: string;
  auth_type: string;
  password: string;
  access_token: string;
  imap_host: string;
  imap_port: number;
}

export interface FolderInfo {
  name: string;
  count: number;
}

export interface CalendarEvent {
  uid: string;
  summary: string;
  dtstart: string;
  dtend: string;
  location: string;
  organizer: string;
  attendees: string[];
  status: string;
  description: string;
}
