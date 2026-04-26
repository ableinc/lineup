import type { Component } from "solid-js";
import type { ScanSummary } from "@/types";

interface Props {
	scan: ScanSummary;
	onView: () => void;
	onDelete: () => void;
}

const fmt = (ts: number) =>
	new Date(ts * 1000).toLocaleString(undefined, {
		dateStyle: "medium",
		timeStyle: "short",
	});

const HistoryCard: Component<Props> = (props) => {
	return (
		<div class="border-b border-neutral-100 dark:border-neutral-900 px-0 py-4 flex items-center justify-between gap-4">
			<div class="min-w-0">
				<p class="text-sm font-mono text-neutral-900 dark:text-neutral-100 truncate">
					{props.scan.repo_path}
				</p>
				<div class="flex items-center gap-4 mt-1 text-xs text-neutral-400">
					<span>{fmt(props.scan.scanned_at)}</span>
					<span>{props.scan.total_structs} structs</span>
					<span>{props.scan.padded_structs} padded</span>
					<span>{props.scan.bytes_saved}B saveable</span>
					<span>{props.scan.target_arch}</span>
				</div>
			</div>
			<div class="flex gap-3 shrink-0">
				<button
					type="button"
					onClick={props.onView}
					class="text-xs font-medium text-neutral-900 dark:text-neutral-100 hover:opacity-60 transition-opacity underline underline-offset-2 cursor-pointer"
				>
					View
				</button>
				<button
					type="button"
					onClick={props.onDelete}
					class="text-xs text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors cursor-pointer"
				>
					Delete
				</button>
			</div>
		</div>
	);
};

export default HistoryCard;
