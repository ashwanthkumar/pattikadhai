import { useState, useEffect } from "react";
import { AppShell } from "@/components/layout/AppShell";
import { SetupWizard } from "@/components/setup/SetupWizard";

function App() {
  const [setupComplete, setSetupComplete] = useState<boolean | null>(null);

  useEffect(() => {
    const stored = localStorage.getItem("pattikadhai_setup_complete");
    setSetupComplete(stored === "true");
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
