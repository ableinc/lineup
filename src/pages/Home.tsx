import { useNavigate } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Component } from "solid-js";
import { createSignal, For } from "solid-js";
import toast from "solid-toast";
import AlertDialog from "@/components/AlertDialog";
import HistoryCard from "@/components/HistoryCard";
import ProgressModal from "@/components/ProgressModal";
import ScanOptionsModal from "@/components/ScanOptionsModal";
import { settings } from "@/store/settings";
import type { ScanOptions, ScanProgressEvent, ScanSummary } from "@/types";

const Home: Component = () => {
	const navigate = useNavigate();
	const [history, setHistory] = createSignal<ScanSummary[]>([]);
	const [showOptions, setShowOptions] = createSignal(false);
	const [selectedPath, setSelectedPath] = createSignal("");
	const [scanning, setScanning] = createSignal(false);
	const [progress, setProgress] = createSignal<ScanProgressEvent>({
		stage: "",
		file: "",
		pct: 0,
	});
	const [showClearConfirm, setShowClearConfirm] = createSignal(false);
	const [pendingDeleteId, setPendingDeleteId] = createSignal<number | null>(
		null,
	);
	const [cancelFn, setCancelFn] = createSignal<(() => void) | null>(null);

	const loadHistory = async () => {
		const h = await invoke<ScanSummary[]>("get_history");
		setHistory(h);
	};

	loadHistory();

	const openFolder = async () => {
		const path = await invoke<string | null>("open_folder_dialog");
		if (path) {
			setSelectedPath(path);
			setShowOptions(true);
		}
	};

	const startScan = async (opts: ScanOptions) => {
		setShowOptions(false);
		setScanning(true);
		setProgress({ stage: "walking", file: "", pct: 0 });

		const unlisten = await listen<ScanProgressEvent>("scan-progress", (e) => {
			setProgress(e.payload);
		});
		setCancelFn(() => unlisten);

		try {
			const summary = await invoke<ScanSummary>("scan_repo", {
				repoPath: selectedPath(),
				opts,
			});
			unlisten();
			setScanning(false);
			await loadHistory();
			navigate(`/scan/${summary.id}`);
		} catch (err: unknown) {
			unlisten();
			setScanning(false);
			if (
				err === "cancelled" ||
				(typeof err === "string" && err.includes("cancelled"))
			) {
				toast.error("Scan cancelled");
			} else {
				toast.error(`Scan failed: ${err}`);
			}
		}
	};

	const cancelScan = async () => {
		await invoke("cancel_scan");
		const fn_ = cancelFn();
		if (fn_) fn_();
		setScanning(false);
		toast.error("Scan cancelled");
	};

	const deleteScan = async (id: number) => {
		await invoke("delete_scan", { scanId: id });
		setPendingDeleteId(null);
		await loadHistory();
	};

	const clearAll = async () => {
		await invoke("clear_history");
		setHistory([]);
		setShowClearConfirm(false);
	};

	return (
		<div class="h-full overflow-y-auto">
			<div class="max-w-3xl mx-auto px-8 py-10">
				{/* Header */}
				<div class="flex items-center justify-between mb-12">
					<div>
						<h1 class="text-xl font-semibold tracking-tight">Lineup</h1>
						<p class="text-neutral-500 text-sm mt-0.5">
							Go struct padding analyzer
						</p>
					</div>
					<div class="flex items-center gap-3">
						<button
							type="button"
							onClick={() => navigate("/settings")}
							aria-label="Settings"
							class="text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors p-1 cursor-pointer"
						>
							<svg
								xmlns="http://www.w3.org/2000/svg"
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1.75"
								stroke-linecap="round"
								stroke-linejoin="round"
								aria-hidden="true"
							>
								<circle cx="12" cy="12" r="3" />
								<path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
							</svg>
						</button>
						<button
							type="button"
							onClick={openFolder}
							class="bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 text-sm font-medium px-4 py-2 hover:opacity-80 transition-opacity cursor-pointer"
						>
							Open Repository
						</button>
					</div>
				</div>

				{history().length > 0 && (
					<div>
						<div class="flex items-center justify-between mb-3">
							<span class="text-xs font-medium uppercase tracking-widest text-neutral-400">
								Recent Scans
							</span>
							<button
								type="button"
								onClick={() => setShowClearConfirm(true)}
								class="text-xs text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors cursor-pointer"
							>
								Clear All
							</button>
						</div>
						<div class="border-t border-neutral-200 dark:border-neutral-800">
							<For each={history()}>
								{(scan) => (
									<HistoryCard
										scan={scan}
										onView={() => navigate(`/scan/${scan.id}`)}
										onDelete={() => setPendingDeleteId(scan.id)}
									/>
								)}
							</For>
						</div>
					</div>
				)}

				{history().length === 0 && (
					<div class="text-center py-28 text-neutral-400">
						<p class="text-base">No scans yet.</p>
						<p class="text-sm mt-1">Open a Go repository to get started.</p>
					</div>
				)}
			</div>

			<ScanOptionsModal
				open={showOptions()}
				repoPath={selectedPath()}
				initialOptions={{
					target_arch: settings.defaultArch,
					ignore_patterns: settings.defaultIgnorePatterns,
				}}
				onStart={startScan}
				onCancel={() => setShowOptions(false)}
			/>

			<ProgressModal
				open={scanning()}
				progress={progress()}
				onCancel={cancelScan}
			/>

			<AlertDialog
				open={showClearConfirm()}
				title="Clear all history?"
				message="This will permanently delete all scan records. This action cannot be undone."
				confirmLabel="Clear All"
				onConfirm={clearAll}
				onCancel={() => setShowClearConfirm(false)}
			/>

			<AlertDialog
				open={pendingDeleteId() !== null}
				title="Delete scan?"
				message="This will permanently delete this scan record. This action cannot be undone."
				confirmLabel="Delete"
				onConfirm={() => deleteScan(pendingDeleteId()!)}
				onCancel={() => setPendingDeleteId(null)}
			/>
		</div>
	);
};

export default Home;
