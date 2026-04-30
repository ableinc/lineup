import { createStore } from "solid-js/store";
import type { AppSettings } from "@/types";

const STORAGE_KEY = "lineup-settings";

const defaultSettings: AppSettings = {
	theme: "dark",
	defaultArch: "amd64",
	defaultIgnorePatterns: [],
	defaultLanguage: "go",
};

function loadSettings(): AppSettings {
	try {
		const raw = localStorage.getItem(STORAGE_KEY);
		if (raw) {
			const parsed = JSON.parse(raw) as Partial<AppSettings>;
			return { ...defaultSettings, ...parsed };
		}
	} catch {
		// ignore parse errors
	}
	return { ...defaultSettings };
}

export const [settings, setSettingsStore] = createStore<AppSettings>(
	loadSettings(),
);

export function updateSettings(patch: Partial<AppSettings>): void {
	if (patch.theme !== undefined) setSettingsStore("theme", patch.theme);
	if (patch.defaultArch !== undefined)
		setSettingsStore("defaultArch", patch.defaultArch);
	if (patch.defaultIgnorePatterns !== undefined)
		setSettingsStore("defaultIgnorePatterns", patch.defaultIgnorePatterns);
	if (patch.defaultLanguage !== undefined)
		setSettingsStore("defaultLanguage", patch.defaultLanguage);
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
	} catch {
		// ignore storage errors
	}
}
