import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useVoiceSettings } from "@/hooks/useVoiceSettings";
import { applyMigrations } from "@/lib/api";

const VOICE_PRESETS = [
  { value: "af_nova", label: "Nova (storytelling)" },
  { value: "bf_emma", label: "Emma (narration)" },
  { value: "af_heart", label: "Heart (warm)" },
  { value: "af_bella", label: "Bella" },
  { value: "af_jessica", label: "Jessica" },
  { value: "af_sarah", label: "Sarah" },
  { value: "af_sky", label: "Sky" },
  { value: "am_adam", label: "Adam" },
  { value: "am_michael", label: "Michael" },
  { value: "bm_george", label: "George" },
  { value: "bf_lily", label: "Lily" },
  { value: "am_echo", label: "Echo" },
];

export function VoiceSettings() {
  const { settings, loading, saving, save } = useVoiceSettings();
  const [voice, setVoice] = useState(settings.tts_voice);
  const [speed, setSpeed] = useState(settings.tts_speed);
  const [saved, setSaved] = useState(false);
  const [migrationStatus, setMigrationStatus] = useState<"running" | "done" | "error" | null>(null);

  useEffect(() => {
    setVoice(settings.tts_voice);
    setSpeed(settings.tts_speed);
  }, [settings]);

  const hasChanges = voice !== settings.tts_voice || speed !== settings.tts_speed;

  const handleSave = async () => {
    await save({
      tts_voice: voice,
      tts_speed: speed,
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-12">
        <p className="text-muted-foreground">Loading settings...</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl p-8">
      <h2 className="mb-6 text-2xl font-bold tracking-tight">Settings</h2>

      <Card>
        <CardHeader>
          <CardTitle>Voice Generation</CardTitle>
          <CardDescription>
            Configure text-to-speech settings using Kokoro-82M.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="voice">
              Voice
            </label>
            <select
              id="voice"
              value={voice}
              onChange={(e) => setVoice(e.target.value)}
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              {VOICE_PRESETS.map((preset) => (
                <option key={preset.value} value={preset.value}>
                  {preset.label}
                </option>
              ))}
            </select>
            <p className="text-xs text-muted-foreground">
              Choose a voice preset. Nova and Emma work best for storytelling.
            </p>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="speed">
              Speed: {speed}x
            </label>
            <input
              id="speed"
              type="range"
              min="0.5"
              max="2.0"
              step="0.1"
              value={speed}
              onChange={(e) => setSpeed(e.target.value)}
              className="w-full"
            />
            <p className="text-xs text-muted-foreground">
              Adjust speech speed. 1.0 is normal.
            </p>
          </div>

          <div className="flex items-center gap-3 pt-2">
            <Button onClick={handleSave} disabled={!hasChanges || saving}>
              {saving ? "Saving..." : "Save"}
            </Button>
            {saved && (
              <span className="text-sm text-green-600">Settings saved</span>
            )}
          </div>
        </CardContent>
      </Card>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Database</CardTitle>
          <CardDescription>
            Apply pending database migrations to ensure settings are up to date.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex items-center gap-3">
          <Button
            variant="outline"
            onClick={async () => {
              setMigrationStatus("running");
              try {
                await applyMigrations();
                setMigrationStatus("done");
                setTimeout(() => setMigrationStatus(null), 2000);
                // Reload voice settings after migration
                window.location.reload();
              } catch {
                setMigrationStatus("error");
                setTimeout(() => setMigrationStatus(null), 3000);
              }
            }}
            disabled={migrationStatus === "running"}
          >
            {migrationStatus === "running" ? "Applying..." : "Apply Migrations"}
          </Button>
          {migrationStatus === "done" && (
            <span className="text-sm text-green-600">Migrations applied</span>
          )}
          {migrationStatus === "error" && (
            <span className="text-sm text-red-600">Migration failed</span>
          )}
        </CardContent>
      </Card>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Setup</CardTitle>
          <CardDescription>
            Re-run the setup wizard to check or install dependencies.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button
            variant="outline"
            onClick={() => {
              localStorage.removeItem("pattikadhai_setup_complete");
              window.location.reload();
            }}
          >
            Re-run Setup Wizard
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
