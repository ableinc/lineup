import type { RouteSectionProps } from "@solidjs/router";
import Footer from "./Footer";

const Layout = (props: RouteSectionProps) => (
	<div
		style={{
			display: "flex",
			"flex-direction": "column",
			"min-height": "100vh",
		}}
	>
		<main style={{ flex: 1, "padding-bottom": "4rem" }}>{props.children}</main>
		<Footer />
	</div>
);
export default Layout;
