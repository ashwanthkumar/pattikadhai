import { useState, useEffect } from "react";
import { AppShell } from "@/components/layout/AppShell";
import { SetupWizard } from "@/components/setup/SetupWizard";
import { checkDependency } from "@/lib/api";

function App() {
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  useEffect(() => {
    async function verifySetup() {
      const stored = localStorage.getItem("pattikadhai_setup_complete");
      if (stored !== "true") {
        setSetupComplete(false);
        return;
      }

      // Even if localStorage says complete, verify critical deps are still present
      try {
        const [ttsModel, espeakNg] = await Promise.all([
          checkDependency("tts_model"),
          checkDependency("espeak_ng"),
        ]);
        if (!ttsModel.installed || !espeakNg.installed) {
          localStorage.removeItem("pattikadhai_setup_complete");
          setSetupComplete(false);
          return;
        }
      } catch {
        // If checks fail (e.g. uv not installed), show wizard
        localStorage.removeItem("pattikadhai_setup_complete");
        setSetupComplete(false);
        return;
      }

      setSetupComplete(true);
    }
    verifySetup();
  }, []);

  if (setupComplete === null) return null;

  if (!setupComplete) {
    return (
      <SetupWizard
        onComplete={() => {
          localStorage.setItem("pattikadhai_setup_complete", "true");
          setSetupComplete(true);
        }}
      />
    );
  }

  return <AppShell />;
}

export default App;
