import { Dialog } from "@kobalte/core/dialog";
import type { Component } from "solid-js";
import { createSignal, For } from "solid-js";
import type { ScanOptions } from "@/types";

interface Props {
	open: boolean;
	repoPath: string;
	initialOptions?: Partial<ScanOptions>;
	onStart: (opts: ScanOptions) => void;
	onCancel: () => void;
}

const ScanOptionsModal: Component<Props> = (props) => {
	const [arch, setArch] = createSignal(
		props.initialOptions?.target_arch ?? "amd64",
	);
	const [patternsText, setPatternsText] = createSignal(
		(props.initialOptions?.ignore_patterns ?? []).join("\n"),
	);
	const [errors, setErrors] = createSignal<string[]>([]);

	const validate = (): string[] => {
		const patterns = patternsText()
			.split("\n")
			.map((p) => p.trim())
			.filter(Boolean);
		const errs: string[] = [];
		for (const p of patterns) {
			try {
				new RegExp(p);
			} catch {
				errs.push(p);
			}
		}
		return errs;
	};

	const submit = () => {
		const errs = validate();
		if (errs.length > 0) {
			setErrors(errs);
			return;
		}
		const patterns = patternsText()
			.split("\n")
			.map((p) => p.trim())
			.filter(Boolean);
		props.onStart({ ignore_patterns: patterns, target_arch: arch() });
	};

	return (
		<Dialog
			open={props.open}
			onOpenChange={(v) => !v && props.onCancel()}
			modal
		>
			<Dialog.Portal>
				<Dialog.Overlay class="fixed inset-0 bg-black/30 dark:bg-black/50 z-40" />
				<div class="fixed inset-0 flex items-center justify-center z-50 p-4">
					<Dialog.Content class="w-full max-w-md bg-white dark:bg-[#0a0a0a] border border-neutral-200 dark:border-neutral-800 p-8 outline-none">
						<Dialog.Title class="text-sm font-semibold text-neutral-900 dark:text-neutral-100">
							Scan Options
						</Dialog.Title>
						<p class="text-xs text-neutral-400 font-mono mt-1 mb-6 truncate">
							{props.repoPath}
						</p>

						{/* Architecture */}
						<div class="mb-5">
							<p class="text-xs font-medium text-neutral-500 uppercase tracking-widest mb-2">
								Target Architecture
							</p>
							<div class="flex gap-0 border border-neutral-200 dark:border-neutral-800 w-fit">
								<button
									type="button"
									onClick={() => setArch("amd64")}
									class={`px-4 py-2 text-sm font-medium transition-colors ${
										arch() === "amd64"
											? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900"
											: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100"
									}`}
								>
									x86_64
								</button>
								<button
									type="button"
									onClick={() => setArch("arm64")}
									class={`px-4 py-2 text-sm font-medium border-l border-neutral-200 dark:border-neutral-800 transition-colors ${
										arch() === "arm64"
											? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900"
											: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100"
									}`}
								>
									ARM64
								</button>
							</div>
						</div>

						{/* Ignore patterns */}
						<div class="mb-6">
							<label
								for="ignore-patterns"
								class="text-xs font-medium text-neutral-500 uppercase tracking-widest block mb-2"
							>
								Ignore Patterns
								<span class="normal-case tracking-normal font-normal ml-2 text-neutral-400">
									— one regex per line
								</span>
							</label>
							<textarea
								id="ignore-patterns"
								value={patternsText()}
								onInput={(e) => {
									setPatternsText(e.currentTarget.value);
									setErrors([]);
								}}
								placeholder={".*\\.pb\\.go$\n.*_test\\.go$"}
								rows={4}
								class="w-full bg-transparent border border-neutral-200 dark:border-neutral-800 px-3 py-2 text-sm font-mono text-neutral-900 dark:text-neutral-100 placeholder-neutral-300 dark:placeholder-neutral-700 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600 resize-none transition-colors"
							/>
							{errors().length > 0 && (
								<div class="text-xs text-neutral-900 dark:text-neutral-100 mt-2 space-y-0.5">
									<p class="font-semibold">Invalid patterns:</p>
									<For each={errors()}>
										{(e) => <p class="font-mono text-neutral-500">• {e}</p>}
									</For>
								</div>
							)}
						</div>

						<div class="flex justify-end gap-3">
							<button
								type="button"
								onClick={props.onCancel}
								class="text-sm text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors px-4 py-2 border border-neutral-200 dark:border-neutral-800 hover:border-neutral-400 dark:hover:border-neutral-600"
							>
								Cancel
							</button>
							<button
								type="button"
								onClick={submit}
								class="text-sm bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 font-medium px-4 py-2 hover:opacity-80 transition-opacity"
							>
								Start Scan
							</button>
						</div>
					</Dialog.Content>
				</div>
			</Dialog.Portal>
		</Dialog>
	);
};

export default ScanOptionsModal;
