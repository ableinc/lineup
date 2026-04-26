import { Route, Router } from "@solidjs/router";
import { createEffect } from "solid-js";
import { Toaster } from "solid-toast";
import Home from "@/pages/Home";
import ScanResults from "@/pages/ScanResults";
import SettingsPage from "@/pages/Settings";
import { settings } from "@/store/settings";
import Layout from "./shared/Layout";

export default function App() {
	createEffect(() => {
		document.documentElement.classList.toggle(
			"dark",
			settings.theme === "dark",
		);
	});

	return (
		<>
			<Router root={Layout}>
				<Route path="/" component={Home} />
				<Route path="/scan/:scanId" component={ScanResults} />
				<Route path="/settings" component={SettingsPage} />
			</Router>
			<Toaster position="bottom-center" />
		</>
	);
}
