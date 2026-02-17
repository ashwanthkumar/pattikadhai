import { CheckCircle2, XCircle, Loader2, Terminal, Play } from "lucide-react";
import { cn } from "@/lib/utils";
import type { DependencyStatus } from "@/types";
import type { InstallLog } from "@/hooks/useDependencyCheck";

interface DependencyStepProps {
  name: string;
  label: string;
  description: string;
  status: DependencyStatus | undefined;
  checking: boolean;
  installing: boolean;
  installLog: InstallLog | null;
  onInstall: () => void;
}

export function DependencyStep({
  label,
  description,
  status,
  checking,
  installing,
  installLog,
  onInstall,
}: DependencyStepProps) {
  const installed = status?.installed ?? false;
  const hasResult = status !== undefined && !checking && !installing;
  const busy = checking || installing;

  return (
    <div
      className={cn(
        "rounded-lg border p-4 transition-colors",
        busy && "border-blue-300 bg-blue-50",
        hasResult && installed && "border-green-300 bg-green-50",
        hasResult && !installed && "border-red-300 bg-red-50",
        !busy && !hasResult && "border-border bg-card",
      )}
    >
      <div className="flex items-start gap-3">
        <div className="mt-0.5">
          {busy ? (
            <Loader2 className="h-5 w-5 animate-spin text-blue-600" />
          ) : hasResult && installed ? (
            <CheckCircle2 className="h-5 w-5 text-green-600" />
          ) : hasResult && !installed ? (
            <XCircle className="h-5 w-5 text-red-600" />
          ) : (
            <div className="h-5 w-5 rounded-full border-2 border-border" />
          )}
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-baseline gap-2">
            <span className="font-semibold text-foreground">{label}</span>
            {hasResult && installed && status?.version && (
              <span className="text-xs text-green-700 font-mono">
                {status.version}
              </span>
            )}
          </div>
          <p className="text-sm text-muted-foreground mt-0.5">{description}</p>

          {installing && (
            <div className="mt-2 text-sm text-blue-700">
              Running install command...
            </div>
          )}

          {hasResult && !installed && status?.install_command && (
            <div className="mt-2 flex items-center gap-2">
              <div className="flex-1 flex items-center gap-2 rounded-md bg-foreground/5 px-3 py-2">
                <Terminal className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                <code className="text-xs font-mono text-foreground/80 break-all">
                  {status.install_command}
                </code>
              </div>
              <button
                type="button"
                onClick={onInstall}
                disabled={installing}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-md px-3 py-2 text-xs font-medium transition-colors shrink-0",
                  "bg-primary text-primary-foreground hover:bg-primary/90 shadow-sm",
                  "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  "disabled:pointer-events-none disabled:opacity-50",
                )}
              >
                <Play className="h-3.5 w-3.5" />
                Run
              </button>
            </div>
          )}

          {installLog && !installing && (
            <div
              className={cn(
                "mt-2 rounded-md border p-2 text-xs font-mono max-h-32 overflow-y-auto",
                installLog.success
                  ? "border-green-200 bg-green-50 text-green-800"
                  : "border-red-200 bg-red-50 text-red-800",
              )}
            >
              <div className="mb-1 font-sans font-medium">
                {installLog.success ? "Install succeeded" : "Install failed"}
              </div>
              <pre className="whitespace-pre-wrap break-all">
                {installLog.output}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
