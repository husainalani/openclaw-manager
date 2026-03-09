import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { ServiceStatus, SystemInfo } from '../lib/tauri';

interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  timestamp: number;
}

interface AppState {
  // Service status (not persisted — fetched live)
  serviceStatus: ServiceStatus | null;
  setServiceStatus: (status: ServiceStatus | null) => void;

  // System information (not persisted — fetched live)
  systemInfo: SystemInfo | null;
  setSystemInfo: (info: SystemInfo | null) => void;

  // UI state
  loading: boolean;
  setLoading: (loading: boolean) => void;

  // Active notifications (ephemeral)
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id' | 'timestamp'>) => void;
  removeNotification: (id: string) => void;

  // Notification history (persisted, last 50)
  notificationHistory: Notification[];
  clearNotificationHistory: () => void;
}

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      // Service status
      serviceStatus: null,
      setServiceStatus: (status) => set({ serviceStatus: status }),

      // System information
      systemInfo: null,
      setSystemInfo: (info) => set({ systemInfo: info }),

      // UI state
      loading: false,
      setLoading: (loading) => set({ loading }),

      // Active notifications
      notifications: [],
      addNotification: (notification) => {
        const newNotif: Notification = {
          ...notification,
          id: Date.now().toString(),
          timestamp: Date.now(),
        };
        set((state) => ({
          notifications: [...state.notifications, newNotif],
          notificationHistory: [newNotif, ...state.notificationHistory].slice(0, 50),
        }));
      },
      removeNotification: (id) =>
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        })),

      // Notification history
      notificationHistory: [],
      clearNotificationHistory: () => set({ notificationHistory: [] }),
    }),
    {
      name: 'openclaw-manager-store',
      // Only persist notification history; live state is always fetched fresh
      partialize: (state) => ({
        notificationHistory: state.notificationHistory,
      }),
    }
  )
);
