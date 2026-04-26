import type { Component } from "solid-js";
import type { StructSummary } from "@/types";

interface Props {
	struct: StructSummary;
	selected: boolean;
	onClick: () => void;
}

const StructCard: Component<Props> = (props) => {
	const saved = () => props.struct.bytes_saved;

	return (
		<button
			type="button"
			onClick={props.onClick}
			class={`w-full text-left border-b border-neutral-100 dark:border-neutral-900 px-4 py-3 transition-colors ${
				props.selected
					? "bg-neutral-900 dark:bg-neutral-100 cursor-default"
					: "hover:bg-neutral-50 dark:hover:bg-neutral-950 cursor-pointer"
			}`}
		>
			<div class="flex items-center justify-between gap-3">
				<div class="min-w-0">
					<div class="flex items-center gap-2 flex-wrap">
						<span
							class={`font-mono text-sm font-semibold ${props.selected ? "text-white dark:text-neutral-900" : "text-neutral-900 dark:text-neutral-100"}`}
						>
							{props.struct.struct_name}
						</span>
						{props.struct.has_generics && (
							<span
								class={`text-xs px-1.5 py-0.5 border ${props.selected ? "border-neutral-600 dark:border-neutral-400 text-neutral-300 dark:text-neutral-600" : "border-neutral-300 dark:border-neutral-700 text-neutral-400"}`}
							>
								~approx
							</span>
						)}
						{props.struct.has_embedded && (
							<span
								class={`text-xs px-1.5 py-0.5 border ${props.selected ? "border-neutral-600 dark:border-neutral-400 text-neutral-300 dark:text-neutral-600" : "border-neutral-300 dark:border-neutral-700 text-neutral-400"}`}
							>
								embedded
							</span>
						)}
					</div>
					<p
						class={`text-xs mt-0.5 ${props.selected ? "text-neutral-400 dark:text-neutral-500" : "text-neutral-400"}`}
					>
						{props.struct.current_size}B → {props.struct.optimal_size}B
					</p>
				</div>
				{saved() > 0 ? (
					<span
						class={`shrink-0 text-xs font-semibold tabular-nums ${props.selected ? "text-neutral-300 dark:text-neutral-600" : "text-neutral-500"}`}
					>
						−{saved()}B
					</span>
				) : (
					<span
						class={`shrink-0 text-xs ${props.selected ? "text-neutral-400 dark:text-neutral-500" : "text-neutral-300 dark:text-neutral-700"}`}
					>
						optimal
					</span>
				)}
			</div>
		</button>
	);
};

export default StructCard;
