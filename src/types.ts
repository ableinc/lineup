export interface ScanSummary {
	id: number;
	repo_path: string;
	scanned_at: number;
	total_structs: number;
	padded_structs: number;
	bytes_saved: number;
	ignore_patterns: string[];
	target_arch: string;
}

export interface StructSummary {
	id: number;
	struct_name: string;
	line_number: number;
	current_size: number;
	optimal_size: number;
	bytes_saved: number;
	has_generics: boolean;
	has_embedded: boolean;
}

export interface FileScanResult {
	file_path: string;
	structs: StructSummary[];
}

export interface StructDetail {
	id: number;
	scan_id: number;
	file_path: string;
	struct_name: string;
	line_number: number;
	current_size: number;
	optimal_size: number;
	bytes_saved: number;
	current_def: string;
	optimized_def: string;
	has_generics: boolean;
	has_embedded: boolean;
}

export interface ScanOptions {
	ignore_patterns: string[];
	target_arch: string;
}

export interface ScanProgressEvent {
	stage: string;
	file: string;
	pct: number;
}

export interface AppSettings {
	theme: "dark" | "light";
	defaultArch: string;
	defaultIgnorePatterns: string[];
}
