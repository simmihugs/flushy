export type DurationCheckStatus = 'no' | 'yes' | 'maybe';

export interface ParsedEvent {
	eventId: string;
	contentId?: string | null;
	title: string;
	startTime: string;
	endTime: string;
	duration: number;
	dbDuration: number;
	displayedStart?: string | null;
	displayedEnd?: string | null;
	errorFront?: unknown;
	errorBack?: unknown;
	hasError: boolean;
	slotStartError: boolean;
	durationError?: DurationCheckStatus;
	xmlString: string;
}

export interface PtsFile {
	name: string;
	path: string;
	si_events: ParsedEvent[];
	va_events: ParsedEvent[];
	si_events_errors: number;
	va_events_errors: number;
	maybe_event_ids?: string[];
}
