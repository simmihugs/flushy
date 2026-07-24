import * as prettier from 'prettier/standalone';
import * as xmlPlugin from '@prettier/plugin-xml';

export async function formatXml(xmlString: string): Promise<string> {
	if (!xmlString) return '';

	try {
		const formatted = await prettier.format(xmlString, {
			parser: 'xml',
			plugins: [xmlPlugin.default || xmlPlugin],
			xmlWhitespaceSensitivity: 'ignore',
			tabWidth: 2,
			printWidth: 160
		});
		return formatted;
	} catch (error) {
		console.error('Prettier XML Formatting Error:', error);
		return xmlString;
	}
}

export async function highlightXml(xmlString: string): Promise<string> {
	if (!xmlString) return '';

	const formatted = await formatXml(xmlString);

	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

	return escaped
		.replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="xml-comment">$1</span>')
		.replace(/(&lt;\/?)([\w:-]+)([^&]*?)(\/?&gt;)/g, (_match, open, tagName, attrs, close) => {
			const highlightedAttrs = attrs.replace(
				/([\w:-]+)=("[^"]*"|'[^']*')/g,
				'<span class="xml-attr">$1</span>=<span class="xml-string">$2</span>'
			);

			return `<span class="xml-bracket">${open}</span><span class="xml-tag">${tagName}</span>${highlightedAttrs}<span class="xml-bracket">${close}</span>`;
		});
}

const pad = (num: number, length = 2) => String(num).padStart(length, '0');

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

/**
 * Formatiert eine ISO-Zeitangabe/Datum in HH:MM:SS:FF (bei 25 fps)
 */
export function formatTime(timeVal: string) {
	if (!timeVal) return '';
	const date = new Date(timeVal);

	if (isNaN(date.getTime())) return String(timeVal);

	const hours = pad(date.getHours());
	const minutes = pad(date.getMinutes());
	const seconds = pad(date.getSeconds());

	// Millisekunden aus dem Date-Objekt in Frames umrechnen (1 Frame = 40ms bei 25fps)
	const ms = date.getMilliseconds();
	const frames = pad(Math.floor(ms / 40));

	return `${hours}:${minutes}:${seconds}.${frames}fps`;
}

/**
 * Formatiert eine Dauer/Range in Millisekunden zu HH:MM:SS:FF (bei 25 fps)
 */
export function formatMilliseconds(ms: number): string {
	if (!ms || isNaN(ms)) return '00:00:00:00';

	const absMs = Math.abs(ms);

	// 1 Frame = 40ms bei 25 fps
	const frames = Math.floor((absMs % 1000) / 40);
	const totalSeconds = Math.floor(absMs / 1000);

	const seconds = totalSeconds % 60;
	const totalMinutes = Math.floor(totalSeconds / 60);

	const minutes = totalMinutes % 60;
	const hours = Math.floor(totalMinutes / 60);

	return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}.${pad(frames)}fps`;
}
