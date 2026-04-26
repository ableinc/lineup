import { useNavigate, useParams } from "@solidjs/router";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Component } from "solid-js";
import {
	createEffect,
	createResource,
	createSignal,
	For,
	onCleanup,
	Show,
} from "solid-js";
import FileTree from "@/components/FileTree";
import ProgressModal from "@/components/ProgressModal";
import ScanOptionsModal from "@/components/ScanOptionsModal";
import StructCard from "@/components/StructCard";
import StructDetailPanel from "@/components/StructDetail";
import type {
	FileScanResult,
	ScanOptions,
	ScanSummary,
	StructDetail,
	StructSummary,
} from "@/types";

const ScanResults: Component = () => {
	const params = useParams<{ scanId: string }>();
	const navigate = useNavigate();

	const [scanMeta] = createResource(
		() => params.scanId,
		async (id) => {
			const history = await invoke<ScanSummary[]>("get_history");
			return history.find((s) => s.id === Number(id)) ?? null;
		},
	);

	const [files] = createResource(
		() => params.scanId,
		(id) => invoke<FileScanResult[]>("get_scan_detail", { scanId: Number(id) }),
	);

	const [selectedFile, setSelectedFile] = createSignal<string | null>(null);
	const [selectedStruct, setSelectedStruct] = createSignal<StructDetail | null>(
		null,
	);

	createEffect(() => {
		const f = files();
		if (f && f.length > 0 && selectedFile() === null) {
			setSelectedFile(f[0].file_path);
		}
	});

	// Resizable pane widths (in px)
	const [leftWidth, setLeftWidth] = createSignal<number>(12 * 16); // smaller default left pane (12rem)
	const [rightWidth, setRightWidth] = createSignal<number>(520); // larger detail panel to shrink list view

	let dragging: {
		type: "left" | "right";
		startX: number;
		startWidth: number;
	} | null = null;

	const onMouseMove = (e: MouseEvent) => {
		if (!dragging) return;
		const dx = e.clientX - dragging.startX;
		if (dragging.type === "left") {
			setLeftWidth(Math.max(120, dragging.startWidth + dx));
		} else {
			setRightWidth(Math.max(200, dragging.startWidth - dx));
		}
	};

	const onMouseUp = () => {
		dragging = null;
		window.removeEventListener("mousemove", onMouseMove);
		window.removeEventListener("mouseup", onMouseUp);
	};

	onCleanup(() => {
		window.removeEventListener("mousemove", onMouseMove);
		window.removeEventListener("mouseup", onMouseUp);
	});
	const [showRescan, setShowRescan] = createSignal(false);
	const [scanning, setScanning] = createSignal(false);
	const [progress, setProgress] = createSignal({ stage: "", file: "", pct: 0 });

	const visibleStructs = () => {
		const f = selectedFile();
		const all = files();
		if (!all) return [];
		if (!f) return all.flatMap((x) => x.structs);
		return all.find((x) => x.file_path === f)?.structs ?? [];
	};

	const selectStruct = async (s: StructSummary) => {
		const detail = await invoke<StructDetail>("get_struct_detail", {
			structId: s.id,
		});
		setSelectedStruct(detail);
	};

	const startRescan = async (opts: ScanOptions) => {
		setShowRescan(false);
		setScanning(true);
		const meta = scanMeta();
		if (!meta) return;

		const unlisten = await listen<{ stage: string; file: string; pct: number }>(
			"scan-progress",
			(e) => {
				setProgress(e.payload);
			},
		);
		try {
			const summary = await invoke<ScanSummary>("scan_repo", {
				repoPath: meta.repo_path,
				opts,
			});
			unlisten();
			setScanning(false);
			navigate(`/scan/${summary.id}`);
		} catch {
			unlisten();
			setScanning(false);
		}
	};

	return (
		<div class="flex flex-col h-full">
			{/* Toolbar */}
			<header class="flex items-center gap-4 px-6 py-3 border-b border-neutral-200 dark:border-neutral-800 shrink-0">
				<button
					type="button"
					onClick={() => navigate("/")}
					class="text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 text-sm transition-colors cursor-pointer"
				>
					← Back
				</button>
				<span class="text-neutral-400 text-xs font-mono truncate flex-1">
					{scanMeta()?.repo_path}
				</span>
				<div class="flex items-center gap-4 text-xs text-neutral-400 tabular-nums">
					<span>{scanMeta()?.total_structs ?? 0} structs</span>
					<span>{scanMeta()?.padded_structs ?? 0} padded</span>
					<span>{scanMeta()?.bytes_saved ?? 0} B saveable</span>
				</div>
				<button
					type="button"
					onClick={() => setShowRescan(true)}
					class="text-xs border border-neutral-200 dark:border-neutral-800 px-3 py-1.5 text-neutral-600 dark:text-neutral-400 hover:border-neutral-400 dark:hover:border-neutral-600 transition-colors cursor-pointer"
				>
					Re-scan
				</button>
			</header>

			<div class="flex flex-1 overflow-hidden">
				{/* Left: FileTree */}
				<aside
					class="border-r border-neutral-200 dark:border-neutral-800 overflow-y-auto shrink-0"
					style={{ width: `${leftWidth()}px`, ["min-width"]: "120px" }}
				>
					<Show
						when={files()}
						fallback={<p class="p-4 text-neutral-400 text-xs">Loading…</p>}
					>
						<FileTree
							files={files() ?? []}
							selectedFile={selectedFile()}
							onSelect={(f) => {
								setSelectedFile(f === selectedFile() ? null : f);
								setSelectedStruct(null);
							}}
						/>
					</Show>
				</aside>

				{/* Divider: left/main */}
				<button
					type="button"
					class="w-2 cursor-col-resize hover:bg-neutral-100 dark:hover:bg-neutral-800 shrink-0 p-0 m-0"
					aria-label="Resize file list"
					onMouseDown={(e) => {
						dragging = {
							type: "left",
							startX: e.clientX,
							startWidth: leftWidth(),
						};
						window.addEventListener("mousemove", onMouseMove);
						window.addEventListener("mouseup", onMouseUp);
					}}
					onKeyDown={(e: KeyboardEvent) => {
						if (e.key === "ArrowLeft")
							setLeftWidth((w) => Math.max(120, w - 12));
						if (e.key === "ArrowRight") setLeftWidth((w) => w + 12);
					}}
				/>

				{/* Center: StructList */}
				<main class="flex-1 overflow-y-auto border-r border-neutral-200 dark:border-neutral-800 min-w-0">
					<For each={visibleStructs()}>
						{(s) => (
							<StructCard
								struct={s}
								selected={selectedStruct()?.id === s.id}
								onClick={() => selectStruct(s)}
							/>
						)}
					</For>
					{visibleStructs().length === 0 && (
						<p class="text-neutral-400 text-xs p-6">
							No structs with padding issues.
						</p>
					)}
				</main>

				{/* Divider: main/right */}
				<button
					type="button"
					class="w-2 cursor-col-resize hover:bg-neutral-100 dark:hover:bg-neutral-800 shrink-0 p-0 m-0"
					aria-label="Resize detail panel"
					onMouseDown={(e) => {
						dragging = {
							type: "right",
							startX: e.clientX,
							startWidth: rightWidth(),
						};
						window.addEventListener("mousemove", onMouseMove);
						window.addEventListener("mouseup", onMouseUp);
					}}
					onKeyDown={(e: KeyboardEvent) => {
						if (e.key === "ArrowLeft")
							setRightWidth((w) => Math.max(200, w - 12));
						if (e.key === "ArrowRight") setRightWidth((w) => w + 12);
					}}
				/>

				{/* Right: StructDetail */}
				<aside
					class="overflow-y-auto shrink-0"
					style={{ width: `${rightWidth()}px`, ["min-width"]: "200px" }}
				>
					<Show
						when={selectedStruct()}
						fallback={
							<div class="flex items-center justify-center h-full text-neutral-400 text-xs">
								Select a struct to view details
							</div>
						}
					>
						{(s) => <StructDetailPanel detail={s()} />}
					</Show>
				</aside>
			</div>

			<ScanOptionsModal
				open={showRescan() && !!scanMeta()}
				repoPath={scanMeta()?.repo_path ?? ""}
				initialOptions={{
					ignore_patterns: scanMeta()?.ignore_patterns ?? [],
					target_arch: scanMeta()?.target_arch ?? "amd64",
				}}
				onStart={startRescan}
				onCancel={() => setShowRescan(false)}
			/>

			<ProgressModal
				open={scanning()}
				progress={progress()}
				onCancel={async () => {
					await invoke("cancel_scan");
					setScanning(false);
				}}
			/>
		</div>
	);
};

export default ScanResults;
