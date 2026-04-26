import { Dialog } from "@kobalte/core/dialog";
import { Progress } from "@kobalte/core/progress";
import type { Component } from "solid-js";
import type { ScanProgressEvent } from "@/types";

interface Props {
	open: boolean;
	progress: ScanProgressEvent;
	onCancel: () => void;
}

const ProgressModal: Component<Props> = (props) => {
	return (
		<Dialog open={props.open} modal>
			<Dialog.Portal>
				<Dialog.Overlay class="fixed inset-0 bg-black/30 dark:bg-black/50 z-40" />
				<div class="fixed inset-0 flex items-center justify-center z-50 p-4">
					<Dialog.Content class="w-full max-w-sm bg-white dark:bg-[#0a0a0a] border border-neutral-200 dark:border-neutral-800 p-8 outline-none">
						<Dialog.Title class="text-sm font-semibold text-neutral-900 dark:text-neutral-100 mb-5">
							Scanning Repository
						</Dialog.Title>
						<div class="space-y-1 mb-4">
							<p class="text-xs text-neutral-500 capitalize">
								{props.progress.stage || "Initializing"}…
							</p>
							{props.progress.file && (
								<p class="text-xs text-neutral-400 dark:text-neutral-600 truncate font-mono">
									{props.progress.file}
								</p>
							)}
						</div>
						<Progress
							value={props.progress.pct}
							minValue={0}
							maxValue={100}
							class="mb-6"
						>
							<Progress.Track class="w-full h-px bg-neutral-200 dark:bg-neutral-800">
								<Progress.Fill class="h-full bg-neutral-900 dark:bg-neutral-100 transition-all duration-150" />
							</Progress.Track>
						</Progress>
						<div class="flex justify-end">
							<button
								type="button"
								onClick={props.onCancel}
								class="text-xs text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors px-3 py-1.5 border border-neutral-200 dark:border-neutral-800 cursor-pointer"
							>
								Cancel
							</button>
						</div>
					</Dialog.Content>
				</div>
			</Dialog.Portal>
		</Dialog>
	);
};

export default ProgressModal;
