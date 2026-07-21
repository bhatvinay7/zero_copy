// Shared types for the PixelPipe app

export type JobStatus = 'queued' | 'uploading' | 'processing' | 'done' | 'error';

export interface Resolution {
  label: string;        // e.g. "4K UHD"
  tag: string;          // e.g. "4K"
  resolution: string;   // e.g. "UHD_4K"
  url: string;          // R2 public URL
}

export interface Job {
  id: string;
  filename: string;
  original_url?: string;
  status: JobStatus;
  progress: number;           // 0–100 overall
  progress_details?: Record<string, number>; // per-resolution progress (e.g. "UHD_4K": 45)
  createdAt: string;          // ISO string
  resolutions?: Resolution[];
  error?: string;
}

export interface UploadFileState {
  id: string;
  name: string;
  size: number;
  type: string;
  progress: number;       // 0–100
  bytesUploaded: number;
  bytesTotal: number;
  status: 'added' | 'uploading' | 'paused' | 'complete' | 'error';
  chunksUploaded: number;
  chunksTotal: number;
  speed?: number;         // bytes/sec
  eta?: number;           // seconds remaining
  error?: string;
}
