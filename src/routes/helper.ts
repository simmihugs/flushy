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

export function formatMilliseconds(ms: Number) {
	if (ms < 0) {
		ms *= -1;
	}
	const milliseconds = ms % 1000;
	const totalSeconds = Math.floor(ms / 1000);

	const seconds = totalSeconds % 60;
	const totalMinutes = Math.floor(totalSeconds / 60);

	const minutes = totalMinutes % 60;
	const hours = Math.floor(totalMinutes / 60);

	const pad = (num, length = 2) => String(num).padStart(length, '0');

	return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}.${pad(milliseconds, 3)}`;
}
