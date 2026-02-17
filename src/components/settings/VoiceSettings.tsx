import { useState, useEffect } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useVoiceSettings } from "@/hooks/useVoiceSettings";

export function VoiceSettings() {
  const { settings, loading, saving, save } = useVoiceSettings();
  const [voice, setVoice] = useState(settings.tts_voice);
  const [seed, setSeed] = useState(settings.tts_seed);
  const [temperature, setTemperature] = useState(settings.tts_temperature);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setVoice(settings.tts_voice);
    setSeed(settings.tts_seed);
    setTemperature(settings.tts_temperature);
  }, [settings]);

  const hasChanges =
    voice !== settings.tts_voice ||
    seed !== settings.tts_seed ||
    temperature !== settings.tts_temperature;

  const handleSave = async () => {
    await save({
      tts_voice: voice,
      tts_seed: seed,
      tts_temperature: temperature,
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
            Configure text-to-speech settings. Using a fixed seed and low
            temperature ensures consistent voice across audio chunks.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="voice">
              Voice
            </label>
            <Input
              id="voice"
              value={voice}
              disabled
              className="bg-muted"
            />
            <p className="text-xs text-muted-foreground">
              Voice name is fixed to the model's default speaker.
            </p>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="seed">
              Random Seed
            </label>
            <Input
              id="seed"
              type="number"
              min={0}
              value={seed}
              onChange={(e) => setSeed(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              Same seed produces the same voice characteristics. Change to get a
              different voice variant.
            </p>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium" htmlFor="temperature">
              Temperature
            </label>
            <Input
              id="temperature"
              type="number"
              min={0}
              max={1}
              step={0.1}
              value={temperature}
              onChange={(e) => setTemperature(e.target.value)}
            />
            <p className="text-xs text-muted-foreground">
              Lower values (0.1-0.3) produce more consistent voice. Higher
              values add more variation.
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
    </div>
  );
}
