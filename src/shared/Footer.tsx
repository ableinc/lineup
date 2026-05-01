import { getVersion } from "@tauri-apps/api/app";
import { createResource } from "solid-js";

export default function Footer() {
	const [version] = createResource(getVersion);

	return (
		<footer class="fixed bottom-0 left-0 right-0 z-50 pointer-events-auto">
			<div class="h-16 bg-white/95 dark:bg-[#0a0a0a]/95 backdrop-blur-sm border-t border-neutral-200 dark:border-neutral-800">
				{/* full-width row so spans can sit at the far left and far right */}
				<div class="w-full flex items-center justify-between text-xs text-neutral-400 h-full px-0">
					<span class="pl-4">Lineup v{version()}</span>
					<span class="pr-4">Type padding analyzer</span>
				</div>
			</div>
		</footer>
	);
}
