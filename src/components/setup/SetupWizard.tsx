import { useEffect } from "react";
import { RefreshCw } from "lucide-react";
import { cn } from "@/lib/utils";
import { useDependencyCheck } from "@/hooks/useDependencyCheck";
import { DEPENDENCY_STEPS } from "@/lib/constants";
import { DependencyStep } from "@/components/setup/DependencyStep";

interface SetupWizardProps {
  onComplete: () => void;
}

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const { statuses, checking, installing, installLogs, install, checkAll } =
    useDependencyCheck();

  const dependencyNames = DEPENDENCY_STEPS.map((step) => step.name);

  const allChecked = dependencyNames.every(
    (name) => statuses[name] !== undefined,
  );
  const allPassed =
    allChecked && dependencyNames.every((name) => statuses[name]?.installed);
  const isChecking = checking !== null;
  const isInstalling = installing !== null;

  useEffect(() => {
    checkAll(dependencyNames);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const handleCheckAgain = () => {
    checkAll(dependencyNames);
  };

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <div className="w-full max-w-lg">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-foreground">
            Welcome to PattiKadhai
          </h1>
          <p className="text-muted-foreground mt-2">
            Let's make sure everything is set up
          </p>
        </div>

        <div className="rounded-lg border border-amber-200 bg-amber-50 dark:border-amber-900 dark:bg-amber-950/30 p-4 mb-4 text-sm text-amber-800 dark:text-amber-200">
          <p className="font-medium">Disk space required: ~4-6 GB total</p>
          <p className="mt-1 text-amber-700 dark:text-amber-300 text-xs">
            Gemma 3 (~2.5 GB) + TTS model (~1.2 GB) + Python env (~1 GB)
          </p>
        </div>

        <div className="rounded-xl border border-border bg-card p-6 shadow-sm">
          <div className="space-y-3">
            {DEPENDENCY_STEPS.map((step) => (
              <DependencyStep
                key={step.name}
                name={step.name}
                label={step.label}
                description={step.description}
                status={statuses[step.name]}
                checking={checking === step.name}
                installing={installing === step.name}
                installLog={installLogs[step.name] ?? null}
                onInstall={() => install(step.name)}
              />
            ))}
          </div>

          <div className="mt-6 flex items-center gap-3">
            <button
              type="button"
              onClick={handleCheckAgain}
              disabled={isChecking || isInstalling}
              className={cn(
                "inline-flex items-center gap-2 rounded-md border border-border px-4 py-2 text-sm font-medium text-foreground transition-colors",
                "hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                "disabled:pointer-events-none disabled:opacity-50",
              )}
            >
              <RefreshCw
                className={cn("h-4 w-4", isChecking && "animate-spin")}
              />
              Check Again
            </button>

            <button
              type="button"
              onClick={onComplete}
              disabled={!allPassed}
              className={cn(
                "ml-auto inline-flex items-center rounded-md px-6 py-2 text-sm font-medium transition-colors",
                "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                "disabled:pointer-events-none disabled:opacity-50",
                allPassed
                  ? "bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm"
                  : "bg-muted text-muted-foreground",
              )}
            >
              Continue
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
