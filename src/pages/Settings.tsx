import { useNavigate } from "@solidjs/router";
import type { Component } from "solid-js";
import { createSignal, For, Show } from "solid-js";
import { settings, updateSettings } from "@/store/settings";
import { getPlaceholderText } from "@/util";

const Settings: Component = () => {
	const navigate = useNavigate();
	const [patternsText, setPatternsText] = createSignal(
		settings.defaultIgnorePatterns.join("\n"),
	);

	const savePatterns = () => {
		const patterns = patternsText()
			.split("\n")
			.map((p) => p.trim())
			.filter(Boolean);
		updateSettings({ defaultIgnorePatterns: patterns });
	};

	return (
		<div class="h-full overflow-y-auto">
			<div class="max-w-xl mx-auto px-8 py-10">
				{/* Header */}
				<div class="flex items-center gap-4 mb-12">
					<button
						type="button"
						onClick={() => navigate("/")}
						class="text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 text-sm transition-colors cursor-pointer"
					>
						← Back
					</button>
					<div>
						<h1 class="text-xl font-semibold tracking-tight">Settings</h1>
					</div>
				</div>

				<div class="space-y-10">
					{/* Appearance */}
					<section>
						<h2 class="text-xs font-medium text-neutral-400 uppercase tracking-widest mb-4">
							Appearance
						</h2>
						<div class="border-t border-neutral-200 dark:border-neutral-800">
							<div class="flex items-center justify-between py-4 border-b border-neutral-100 dark:border-neutral-900">
								<div>
									<p class="text-sm text-neutral-900 dark:text-neutral-100">
										Theme
									</p>
									<p class="text-xs text-neutral-400 mt-0.5">
										Dark or light interface
									</p>
								</div>
								<div class="flex gap-0 border border-neutral-200 dark:border-neutral-800">
									<button
										type="button"
										onClick={() => updateSettings({ theme: "light" })}
										class={`px-4 py-2 text-sm font-medium transition-colors ${
											settings.theme === "light"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										Light
									</button>
									<button
										type="button"
										onClick={() => updateSettings({ theme: "dark" })}
										class={`px-4 py-2 text-sm font-medium border-l border-neutral-200 dark:border-neutral-800 transition-colors ${
											settings.theme === "dark"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										Dark
									</button>
								</div>
							</div>
						</div>
					</section>

					{/* Scan Defaults */}
					<section>
						<h2 class="text-xs font-medium text-neutral-400 uppercase tracking-widest mb-4">
							Scan Defaults
						</h2>
						<div class="border-t border-neutral-200 dark:border-neutral-800">
							{" "}
							{/* Default Language */}
							<div class="flex items-center justify-between py-4 border-b border-neutral-100 dark:border-neutral-900">
								<div>
									<p class="text-sm text-neutral-900 dark:text-neutral-100">
										Default Language
									</p>
									<p class="text-xs text-neutral-400 mt-0.5">
										Pre-selected when starting a new scan
									</p>
								</div>
								<div class="flex gap-0 border border-neutral-200 dark:border-neutral-800">
									<button
										type="button"
										onClick={() => updateSettings({ defaultLanguage: "GO" })}
										class={`px-4 py-2 text-sm font-medium transition-colors ${
											settings.defaultLanguage === "GO"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										Go
									</button>
									<button
										type="button"
										onClick={() => updateSettings({ defaultLanguage: "TS" })}
										class={`px-4 py-2 text-sm font-medium border-l border-neutral-200 dark:border-neutral-800 transition-colors ${
											settings.defaultLanguage === "TS"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										TypeScript
									</button>
								</div>
							</div>
							{/* Default Architecture */}
							<div class="flex items-center justify-between py-4 border-b border-neutral-100 dark:border-neutral-900">
								<div>
									<p class="text-sm text-neutral-900 dark:text-neutral-100">
										Default Architecture
									</p>
									<p class="text-xs text-neutral-400 mt-0.5">
										Pre-selected when starting a new scan
									</p>
								</div>
								<div class="flex gap-0 border border-neutral-200 dark:border-neutral-800">
									<button
										type="button"
										onClick={() => updateSettings({ defaultArch: "amd64" })}
										class={`px-4 py-2 text-sm font-medium transition-colors ${
											settings.defaultArch === "amd64"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										x86_64
									</button>
									<button
										type="button"
										onClick={() => updateSettings({ defaultArch: "arm64" })}
										class={`px-4 py-2 text-sm font-medium border-l border-neutral-200 dark:border-neutral-800 transition-colors ${
											settings.defaultArch === "arm64"
												? "bg-neutral-900 dark:bg-neutral-100 text-white dark:text-neutral-900 cursor-default"
												: "text-neutral-500 hover:text-neutral-900 dark:hover:text-neutral-100 cursor-pointer"
										}`}
									>
										ARM64
									</button>
								</div>
							</div>
							{/* Default Ignore Patterns */}
							<div class="py-4 space-y-3 border-b border-neutral-100 dark:border-neutral-900">
								<div>
									<p class="text-sm text-neutral-900 dark:text-neutral-100">
										Default Ignore Patterns
									</p>
									<p class="text-xs text-neutral-400 mt-0.5">
										Applied to new scans — one regex per line
									</p>
								</div>
								<textarea
									value={patternsText()}
									onInput={(e) => setPatternsText(e.currentTarget.value)}
									onBlur={savePatterns}
									placeholder={getPlaceholderText(settings.defaultLanguage)}
									rows={5}
									class="w-full bg-transparent border border-neutral-200 dark:border-neutral-800 px-3 py-2 text-sm text-neutral-900 dark:text-neutral-100 font-mono placeholder-neutral-300 dark:placeholder-neutral-700 focus:outline-none focus:border-neutral-400 dark:focus:border-neutral-600 resize-none transition-colors"
									autocomplete="off"
									spellcheck="false"
									autocorrect="off"
								/>
								<p class="text-xs text-neutral-400">
									Saved automatically on blur.
								</p>
								{/* Saved patterns grid */}
								<Show when={settings.defaultIgnorePatterns.length > 0}>
									<div class="flex flex-wrap gap-2 pt-1">
										<For each={settings.defaultIgnorePatterns}>
											{(pattern) => (
												<span class="inline-flex items-center gap-1.5 border border-neutral-200 dark:border-neutral-800 px-2 py-1 text-xs font-mono text-neutral-600 dark:text-neutral-400">
													{pattern}
													<button
														type="button"
														aria-label={`Remove ${pattern}`}
														onClick={() => {
															const next =
																settings.defaultIgnorePatterns.filter(
																	(p) => p !== pattern,
																);
															updateSettings({ defaultIgnorePatterns: next });
															setPatternsText(next.join("\n"));
														}}
														class="text-neutral-400 hover:text-neutral-900 dark:hover:text-neutral-100 transition-colors leading-none cursor-pointer"
													>
														x
													</button>
												</span>
											)}
										</For>
									</div>
								</Show>
							</div>
						</div>
					</section>
				</div>
			</div>
		</div>
	);
};

export default Settings;
