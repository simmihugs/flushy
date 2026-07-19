export function formatTime(timeVal: string) {
	if (!timeVal) return '';
	const date = new Date(timeVal);
	return isNaN(date.getTime())
		? String(timeVal)
		: date.toLocaleTimeString([], {
				hour: '2-digit',
				minute: '2-digit',
				second: '2-digit',
				fractionalSecondDigits: 3
			});
}
