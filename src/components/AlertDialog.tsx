import { AlertDialog as KobalteAlertDialog } from "@kobalte/core/alert-dialog";
import type { Component } from "solid-js";

interface Props {
	open: boolean;
	title: string;
	message: string;
	confirmLabel?: string;
	onConfirm: () => void;
	onCancel: () => void;
}

const AlertDialog: Component<Props> = (props) => {
	return (
		<KobalteAlertDialog
			open={props.open}
			onOpenChange={(v) => !v && props.onCancel()}
		>
			<KobalteAlertDialog.Portal>
				<KobalteAlertDialog.Overlay class="fixed inset-0 bg-black/30 dark:bg-black/50 z-40" />
				<div class="fixed inset-0 flex items-center justify-center z-50 p-4">
					<KobalteAlertDialog.Content class="w-full max-w-sm bg-white dark:bg-[#0a0a0a] border border-neutral-200 dark:border-neutral-800 p-8 outline-none">
						<KobalteAlertDialog.Title class="text-sm font-semibold text-neutral-900 dark:text-neutral-100 mb-2">
							{props.title}
						</KobalteAlertDialog.Title>
						<KobalteAlertDialog.Description class="text-sm text-neutral-500 mb-6">
							{props.message}
						</KobalteAlertDialog.Description>
						<div class="flex justify-end gap-3">
							<button
								type="button"
								onClick={props.onCancel}
								class="text-sm text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors px-4 py-2 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-400 dark:hover:border-neutral-600 cursor-pointer"
							>
								Cancel
							</button>
							<button
								type="button"
								onClick={props.onConfirm}
								class="text-sm bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 font-medium px-4 py-2 hover:opacity-80 transition-opacity cursor-pointer"
							>
								{props.confirmLabel ?? "Confirm"}
							</button>
						</div>
					</KobalteAlertDialog.Content>
				</div>
			</KobalteAlertDialog.Portal>
		</KobalteAlertDialog>
	);
};

export default AlertDialog;
