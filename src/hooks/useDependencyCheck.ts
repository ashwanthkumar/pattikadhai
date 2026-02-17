import { useState, useCallback } from "react";
import { checkDependency, installDependency } from "@/lib/api";
import type { DependencyStatus } from "@/types";

export interface InstallLog {
  name: string;
  success: boolean;
  output: string;
}

export function useDependencyCheck() {
  const [statuses, setStatuses] = useState<Record<string, DependencyStatus>>(
    {},
  );
  const [checking, setChecking] = useState<string | null>(null);
  const [installing, setInstalling] = useState<string | null>(null);
  const [installLogs, setInstallLogs] = useState<Record<string, InstallLog>>(
    {},
  );

  const check = useCallback(async (name: string) => {
    setChecking(name);
    try {
      const status = await checkDependency(name);
      setStatuses((prev) => ({ ...prev, [name]: status }));
      return status;
    } catch (e) {
      const errorStatus: DependencyStatus = {
        name,
        installed: false,
        version: null,
        install_command: "Check error logs",
      };
      setStatuses((prev) => ({ ...prev, [name]: errorStatus }));
      return errorStatus;
    } finally {
      setChecking(null);
    }
  }, []);

  const install = useCallback(
    async (name: string) => {
      setInstalling(name);
      // Clear previous log for this dep
      setInstallLogs((prev) => {
        const next = { ...prev };
        delete next[name];
        return next;
      });
      try {
        const result = await installDependency(name);
        setInstallLogs((prev) => ({
          ...prev,
          [name]: { name, success: result.success, output: result.output },
        }));
        // Re-check after install attempt
        await check(name);
        return result;
      } catch (e) {
        const output = e instanceof Error ? e.message : String(e);
        setInstallLogs((prev) => ({
          ...prev,
          [name]: { name, success: false, output },
        }));
        return { success: false, output };
      } finally {
        setInstalling(null);
      }
    },
    [check],
  );

  const checkAll = useCallback(
    async (names: string[]) => {
      for (const name of names) {
        await check(name);
      }
    },
    [check],
  );

  return {
    statuses,
    checking,
    installing,
    installLogs,
    check,
    install,
    checkAll,
  };
}
