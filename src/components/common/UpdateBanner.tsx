import { useState, useEffect } from 'react';
import { check, Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { Button } from './Button';

export function UpdateBanner() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [checking, setChecking] = useState(true);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    checkForUpdate();
  }, []);

  async function checkForUpdate() {
    try {
      setChecking(true);
      const available = await check();
      if (available) {
        setUpdate(available);
      }
    } catch (e) {
      console.error('Failed to check for updates:', e);
    } finally {
      setChecking(false);
    }
  }

  async function downloadAndInstall() {
    if (!update) return;

    try {
      setDownloading(true);
      setProgress(0);

      let contentLength = 0;
      let downloaded = 0;

      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            contentLength = event.data.contentLength ?? 0;
            downloaded = 0;
            setProgress(0);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            if (contentLength > 0) {
              setProgress(Math.round((downloaded / contentLength) * 100));
            }
            break;
          case 'Finished':
            setProgress(100);
            break;
        }
      });

      // Relaunch the app after installation
      await relaunch();
    } catch (e) {
      console.error('Failed to download update:', e);
      setDownloading(false);
    }
  }

  // Don't show anything if checking, no update, or dismissed
  if (checking || !update || dismissed) {
    return null;
  }

  return (
    <div className="bg-amber-600 text-amber-950 px-4 py-2 flex items-center justify-between">
      <div className="flex items-center gap-2">
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        <span className="font-medium">
          {downloading
            ? `Downloading update... ${progress}%`
            : `Version ${update.version} is available!`
          }
        </span>
      </div>
      <div className="flex items-center gap-2">
        {!downloading && (
          <>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => setDismissed(true)}
            >
              Later
            </Button>
            <Button
              size="sm"
              onClick={downloadAndInstall}
            >
              Update Now
            </Button>
          </>
        )}
        {downloading && (
          <div className="w-32 bg-amber-800 rounded-full h-2">
            <div
              className="bg-amber-300 h-2 rounded-full transition-all duration-200"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}
      </div>
    </div>
  );
}
