import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface MicrophoneData {
  level: number; // in dB
  rms: number; // raw RMS value
  sample_rate: number;
  channels: number;
}

export class MicrophoneMonitor {
  private unlistenFn: UnlistenFn | null = null;
  private isMonitoring = false;

  constructor(private onLevelChange?: (level: number, rms: number) => void) {}

  async startMonitoring(): Promise<void> {
    if (this.isMonitoring) return;

    try {
      // Setup listener first
      this.unlistenFn = await listen<MicrophoneData>('microphone-data', (event) => {
        const data = event.payload;

        // Call the callback if provided
        if (this.onLevelChange) {
          this.onLevelChange(data.level, data.rms);
        }

        // You can also implement visualization logic here or in your UI component
      });

      // Then start the microphone monitoring on the backend
      await invoke('settings_microphone', { active: true });
      this.isMonitoring = true;

      console.log('Microphone monitoring started');
    } catch (error) {
      console.error('Error starting microphone monitoring:', error);
      this.stopMonitoring(); // Clean up if there was an error
      throw error;
    }
  }

  async stopMonitoring(): Promise<void> {
    if (!this.isMonitoring) return;

    try {
      // First stop the backend
      await invoke('settings_microphone', { active: false });

      // Then remove the listener
      if (this.unlistenFn) {
        await this.unlistenFn();
        this.unlistenFn = null;
      }

      this.isMonitoring = false;
      console.log('Microphone monitoring stopped');
    } catch (error) {
      console.error('Error stopping microphone monitoring:', error);
      throw error;
    }
  }

  isActive(): boolean {
    return this.isMonitoring;
  }
}
