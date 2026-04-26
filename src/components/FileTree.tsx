import type { Component } from "solid-js";
import { For } from "solid-js";
import type { FileScanResult } from "@/types";

interface Props {
	files: FileScanResult[];
	selectedFile: string | null;
	onSelect: (path: string) => void;
}

const FileTree: Component<Props> = (props) => {
	const fileName = (path: string) => path.split("/").pop() ?? path;
	const dirName = (path: string) => {
		const parts = path.split("/");
		return parts.slice(0, -1).join("/");
	};

	return (
		<div class="py-2">
			<p class="px-4 py-2 text-xs font-medium text-neutral-400 uppercase tracking-widest">
				Files
			</p>
			<For each={props.files}>
				{(file) => {
					const issueCount = file.structs.filter(
						(s) => s.bytes_saved > 0,
					).length;
					const isSelected = props.selectedFile === file.file_path;
					return (
						<button
							type="button"
							onClick={() => props.onSelect(file.file_path)}
							class={`w-full text-left px-4 py-2.5 flex items-center justify-between gap-2 transition-colors border-b border-neutral-100 dark:border-neutral-900 ${
								isSelected
									? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900"
									: "text-neutral-600 dark:text-neutral-400 hover:bg-neutral-50 dark:hover:bg-neutral-950"
							}`}
						>
							<div class="min-w-0">
								<p class="text-xs font-mono truncate">
									{fileName(file.file_path)}
								</p>
								<p
									class={`text-xs truncate ${isSelected ? "text-neutral-400 dark:text-neutral-500" : "text-neutral-300 dark:text-neutral-700"}`}
								>
									{dirName(file.file_path)}
								</p>
							</div>
							{issueCount > 0 && (
								<span
									class={`shrink-0 text-xs font-medium tabular-nums ${isSelected ? "text-neutral-300 dark:text-neutral-600" : "text-neutral-400"}`}
								>
									{issueCount}
								</span>
							)}
						</button>
					);
				}}
			</For>
		</div>
	);
};

export default FileTree;
