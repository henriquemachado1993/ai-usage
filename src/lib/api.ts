import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Account, DockState, NewAccount, Settings, UsageSnapshot } from "./types";

export const api = {
  getSnapshots: () => invoke<UsageSnapshot[]>("get_snapshots"),
  refreshNow: () => invoke<void>("refresh_now"),
  listAccounts: () => invoke<Account[]>("list_accounts"),
  addAccount: (input: NewAccount) => invoke<Account>("add_account", { input }),
  removeAccount: (id: string) => invoke<void>("remove_account", { id }),
  renameAccount: (id: string, label: string | null) => invoke<void>("rename_account", { id, label }),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<Settings>("set_settings", { settings }),
  redetectAccounts: () => invoke<void>("redetect_accounts"),
  quitApp: () => invoke<void>("quit_app"),
  getDock: () => invoke<DockState>("get_dock"),
  setCollapsed: (collapsed: boolean, stripHeight: number) =>
    invoke<DockState>("set_collapsed", { collapsed, stripHeight }),
};

export const onUsageUpdated = (cb: (s: UsageSnapshot[]) => void) =>
  listen<UsageSnapshot[]>("usage-updated", (e) => cb(e.payload));

export const onRefreshing = (cb: (busy: boolean) => void) =>
  listen<boolean>("refreshing", (e) => cb(e.payload));

export const onOpenSettings = (cb: () => void) => listen("open-settings", () => cb());

export const onDockChanged = (cb: (d: DockState) => void) =>
  listen<DockState>("dock-changed", (e) => cb(e.payload));
