/** Application settings */
export interface AppSettings {
  esoAddonPath?: string;
  checkUpdatesOnStartup: boolean;
  autoUpdate: boolean;
  theme: 'system' | 'light' | 'dark';
  indexUrl?: string;
}
