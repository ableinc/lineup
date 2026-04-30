import type { Component } from "solid-js";
import { createSignal } from "solid-js";
import toast from "solid-toast";
import type { StructDetail } from "@/types";

interface Props {
	detail: StructDetail;
}

const StructDetailPanel: Component<Props> = (props) => {
	const [copied, setCopied] = createSignal(false);

	const copy = async () => {
		await navigator.clipboard.writeText(props.detail.optimized_def);
		setCopied(true);
		setTimeout(() => setCopied(false), 1500);
		toast.success("Copied to clipboard");
	};

	return (
		<div class="p-6 space-y-5">
			{/* Name + location */}
			<div>
				<div class="flex items-center gap-2 flex-wrap mb-1">
					<h2 class="font-mono font-semibold text-sm text-neutral-900 dark:text-neutral-100">
						{props.detail.struct_name}
					</h2>{" "}
					{props.detail.declaration_kind !== "struct" && (
						<span class="text-xs border border-neutral-300 dark:border-neutral-700 px-1.5 py-0.5 text-neutral-400">
							{props.detail.declaration_kind}
						</span>
					)}{" "}
					{props.detail.has_generics && (
						<span class="text-xs border border-neutral-300 dark:border-neutral-700 px-1.5 py-0.5 text-neutral-400">
							~approx
						</span>
					)}
					{props.detail.has_embedded && (
						<span class="text-xs border border-neutral-300 dark:border-neutral-700 px-1.5 py-0.5 text-neutral-400">
							embedded
						</span>
					)}
				</div>
				<p class="text-xs text-neutral-400 font-mono">
					{props.detail.file_path}:{props.detail.line_number}
				</p>
			</div>

			{/* Size stats */}
			<div class="grid grid-cols-3 border border-neutral-200 dark:border-neutral-800 divide-x divide-neutral-200 dark:divide-neutral-800">
				<div class="px-4 py-3">
					<p class="text-xs text-neutral-400 mb-1">Current</p>
					<p class="text-sm font-semibold font-mono text-neutral-900 dark:text-neutral-100">
						{props.detail.current_size}B
					</p>
				</div>
				<div class="px-4 py-3">
					<p class="text-xs text-neutral-400 mb-1">Optimal</p>
					<p class="text-sm font-semibold font-mono text-neutral-900 dark:text-neutral-100">
						{props.detail.optimal_size}B
					</p>
				</div>
				<div class="px-4 py-3">
					<p class="text-xs text-neutral-400 mb-1">Saved</p>
					<p class="text-sm font-semibold font-mono text-neutral-900 dark:text-neutral-100">
						{props.detail.bytes_saved}B
					</p>
				</div>
			</div>

			{/* Code comparison */}
			<div class="grid grid-cols-2 gap-4">
				<div>
					<p class="text-xs font-medium text-neutral-400 uppercase tracking-widest mb-2">
						Current
					</p>
					<pre class="text-xs border border-neutral-200 dark:border-neutral-800 p-3 overflow-x-auto text-neutral-600 dark:text-neutral-400 whitespace-pre-wrap break-all leading-5 font-mono bg-neutral-50 dark:bg-neutral-950">
						{props.detail.current_def}
					</pre>
				</div>
				<div>
					<div class="flex items-center justify-between mb-2">
						<p class="text-xs font-medium text-neutral-400 uppercase tracking-widest">
							Optimized
						</p>
						<button
							type="button"
							onClick={copy}
							class="text-xs text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors underline underline-offset-2 cursor-pointer"
						>
							{copied() ? "Copied" : "Copy"}
						</button>
					</div>
					<pre class="text-xs border border-neutral-200 dark:border-neutral-800 p-3 overflow-x-auto text-neutral-900 dark:text-neutral-100 whitespace-pre-wrap break-all leading-5 font-mono bg-neutral-50 dark:bg-neutral-950">
						{props.detail.optimized_def}
					</pre>
				</div>
			</div>
		</div>
	);
};

export default StructDetailPanel;
