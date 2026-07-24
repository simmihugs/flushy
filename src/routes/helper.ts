export function sendeTag(timeVal: string) {
	if (!timeVal) return '';
	const date = new Date(timeVal);

	return isNaN(date.getTime())
		? String(timeVal)
		: date.toLocaleDateString('de-DE', {
				day: '2-digit',
				month: '2-digit',
				year: 'numeric'
			});
}

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

export function formatMilliseconds(ms: number): string {
	if (!ms || isNaN(ms)) return '00:00:00.000';

	const absMs = Math.abs(ms);

	const milliseconds = Math.floor(absMs % 1000);
	const totalSeconds = Math.floor(absMs / 1000);

	const seconds = totalSeconds % 60;
	const totalMinutes = Math.floor(totalSeconds / 60);

	const minutes = totalMinutes % 60;
	const hours = Math.floor(totalMinutes / 60);

	const pad = (num: number, length = 2) => String(num).padStart(length, '0');

	return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}.${pad(milliseconds, 3)}`;
}

export function formatXml(xmlString: string): string {
	if (!xmlString) return '';
	let formatted = '';
	let indent = '';
	const tab = '  ';
	xmlString.split(/>\s*</).forEach((node) => {
		if (node.match(/^\/\w/)) {
			indent = indent.substring(tab.length);
		}

		formatted += indent + '<' + node + '>\n';

		if (node.match(/^<?\w[^>]*[^\/]$/) && !node.startsWith('input')) {
			indent += tab;
		}
	});

	return formatted.substring(1, formatted.length - 2).trim();
}

export function highlightXml(xmlString: string): string {
	if (!xmlString) return '';

	const formatted = formatXml(xmlString);
	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

	return escaped
		.replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="xml-comment">$1</span>')
		.replace(/(&lt;\/?)([\w:-]+)([^&]*?)(&gt;)/g, (_match, open, tagName, attrs, close) => {
			const highlightedAttrs = attrs.replace(
				/([\w:-]+)=("[^"]*"|'[^']*')/g,
				'<span class="xml-attr">$1</span>=<span class="xml-string">$2</span>'
			);
			return `${open}<span class="xml-tag">${tagName}</span>${highlightedAttrs}${close}`;
		});
}
