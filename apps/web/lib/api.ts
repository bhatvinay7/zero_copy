import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useEffect } from 'react';
import type { Job } from './types';

const BASE_URL = process.env.NEXT_PUBLIC_API_URL! || 'http://localhost:3001/api';

function getHeaders() {
  const headers: HeadersInit = {
    'Content-Type': 'application/json',
  };
  if (typeof window !== 'undefined') {
    const token = localStorage.getItem('jwt_token');
    if (token) {
      headers['authorization'] = `Bearer ${token}`;
    }
  }
  return headers;
}

export function useVideos(token: string | null) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['videos'],
    queryFn: async (): Promise<Job[]> => {
      const res = await fetch(`${BASE_URL}/videos`, { headers: getHeaders() });
      if (!res.ok) throw new Error('Failed to fetch videos');
      return res.json();
    },
    enabled: !!token,
  });

  useEffect(() => {
    if (!token) return;
    const sseBase = process.env.NEXT_PUBLIC_SSE_ENDPOINT || 'http://localhost:3001/api/videos/events';
    const eventSource = new EventSource(`${sseBase}?token=${token}`);

    eventSource.addEventListener('progress', (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data);
        const videoId = data.video_id.toString();
        const percentage = data.percentage;
        const resolutions = data.resolutions || {};

        queryClient.setQueryData<Job[]>(['videos'], (old) => {
          if (!old) return old;
          return old.map((job) => {
            if (job.id === videoId) {
              return {
                ...job,
                progress: percentage,
                progress_details: {
                  ...job.progress_details,
                  ...resolutions,
                },
                status: percentage >= 100 ? 'done' : 'processing',
              };
            }
            return job;
          });
        });
      } catch (err) {
        console.error('Failed to parse progress SSE event:', err);
      }
    });

    return () => {
      eventSource.close();
    };
  }, [token, queryClient]);

  return query;
}

export function useCancelVideo() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const res = await fetch(`${BASE_URL}/videos/${id}/cancel`, {
        method: 'POST',
        headers: getHeaders(),
      });
      if (!res.ok) throw new Error('Failed to cancel video');
      return res.json();
    },
    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey: ['videos'] });
      const previousVideos = queryClient.getQueryData<Job[]>(['videos']);
      queryClient.setQueryData<Job[]>(['videos'], (old) => {
        if (!old) return old;
        return old.map(v => v.id === id ? { ...v, status: 'error', error: 'Transcode cancelled by user' } : v);
      });
      return { previousVideos };
    },
    onError: (err, id, context) => {
      if (context?.previousVideos) {
        queryClient.setQueryData(['videos'], context.previousVideos);
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ['videos'] });
    },
  });
}

export function useLogin() {
  return useMutation({
    mutationFn: async (credentials: any) => {
      const res = await fetch(`${BASE_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(credentials),
      });
      let data;
      try {
        data = await res.json();
      } catch (err) {
        throw new Error('Server returned an invalid response. Please try again later.');
      }
      if (!res.ok || !data.success) {
        throw new Error(data.error || 'Login failed. Please check your credentials.');
      }
      return data;
    },
  });
}

export function useSignup() {
  return useMutation({
    mutationFn: async (credentials: any) => {
      const res = await fetch(`${BASE_URL}/auth/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(credentials),
      });
      let data;
      try {
        data = await res.json();
      } catch (err) {
        throw new Error('Server returned an invalid response. Please try again later.');
      }
      if (!res.ok || !data.success) {
        throw new Error(data.error || 'Signup failed. Please try again.');
      }
      return data;
    },
  });
}
