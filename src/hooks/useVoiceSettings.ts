import { useState, useEffect, useCallback } from "react";
import {
  getVoiceSettings,
  updateSetting,
  type VoiceSettingsData,
} from "@/lib/database";

export function useVoiceSettings() {
  const [settings, setSettings] = useState<VoiceSettingsData>({
    tts_voice: "Luna",
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    getVoiceSettings()
      .then(setSettings)
      .finally(() => setLoading(false));
  }, []);

  const save = useCallback(
    async (updated: VoiceSettingsData) => {
      setSaving(true);
      try {
        const keys = Object.keys(updated) as (keyof VoiceSettingsData)[];
        for (const key of keys) {
          if (updated[key] !== settings[key]) {
            await updateSetting(key, updated[key]);
          }
        }
        setSettings(updated);
      } finally {
        setSaving(false);
      }
    },
    [settings],
  );

  return { settings, loading, saving, save };
}
