// import * as prettier from 'prettier/standalone';
// import * as xmlPlugin from '@prettier/plugin-xml';

// async function formatXml(xmlString: string): string {
// 	if (!xmlString) return '';

// 	try {
// 		const formatted = await prettier.format(xmlString, {
// 			parser: 'xml',
// 			plugins: [xmlPlugin],
// 			xmlWhitespaceSensitivity: 'ignore',
// 			tabWidth: 2,
// 			printWidth: 80
// 		});
// 		return formatted;
// 	} catch (error) {
// 		console.error('Prettier XML Formatting Error:', error);
// 		return xmlString;
// 	}
// }

// export async function formatXml(xmlString: string): Promise<string> {
// 	if (!xmlString) return '';

// 	try {
// 		const formatted = await prettier.format(xmlString, {
// 			parser: 'xml',
// 			plugins: [xmlPlugin],
// 			xmlWhitespaceSensitivity: 'ignore',
// 			tabWidth: 2,
// 			printWidth: 160 // <-- ERHÖHT: Verhindert unnötige Umbrüche & die riesige Gap links!
// 		});
// 		return formatted;
// 	} catch (error) {
// 		console.error('Prettier XML Formatting Error:', error);
// 		return xmlString;
// 	}
// }

// export async function highlightXml(xmlString: string): Promise<string> {
// 	if (!xmlString) return '';

// 	const formatted = await formatXml(xmlString);

// 	// 1. HTML Escapen
// 	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// 	// 2. Highlighting: Klammern bekommen eigene Klasse .xml-bracket!
// 	return escaped
// 		.replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="xml-comment">$1</span>')
// 		.replace(/(&lt;\/?)([\w:-]+)([^&]*?)(\/?&gt;)/g, (_match, open, tagName, attrs, close) => {
// 			const highlightedAttrs = attrs.replace(
// 				/([\w:-]+)=("[^"]*"|'[^']*')/g,
// 				'<span class="xml-attr">$1</span>=<span class="xml-string">$2</span>'
// 			);

// 			// Open & Close bekommen .xml-bracket
// 			return `<span class="xml-bracket">${open}</span><span class="xml-tag">${tagName}</span>${highlightedAttrs}<span class="xml-bracket">${close}</span>`;
// 		});
// }

import * as prettier from 'prettier/standalone';
import * as xmlPlugin from '@prettier/plugin-xml';

export async function formatXml(xmlString: string): Promise<string> {
	if (!xmlString) return '';

	try {
		const formatted = await prettier.format(xmlString, {
			parser: 'xml',
			// Prettier verlangt hier das Plugin-Objekt (oder dessen default/Module)
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

	// 1. HTML Escapen
	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

	// 2. Highlighting mit separaten Brackets
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

// export async function highlightXml(xmlString: string): Promise<string> {
// 	if (!xmlString) return '';

// 	const formatted = await formatXml(xmlString);

// 	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// 	return escaped
// 		.replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="xml-comment">$1</span>')
// 		.replace(/(&lt;\/?)([\w:-]+)([^&]*?)(&gt;)/g, (_match, open, tagName, attrs, close) => {
// 			const highlightedAttrs = attrs.replace(
// 				/([\w:-]+)=("[^"]*"|'[^']*')/g,
// 				'<span class="xml-attr">$1</span>=<span class="xml-string">$2</span>'
// 			);
// 			return `${open}<span class="xml-tag">${tagName}</span>${highlightedAttrs}${close}`;
// 		});
// }
// export async function highlightXml(xmlString: string): Promise<string> {
// 	if (!xmlString) return '';

// 	const formatted = await formatXml(xmlString);

// 	// 1. HTML Escapen
// 	const escaped = formatted.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

// 	// 2. Highlighting mit Klammern (&lt; und &gt;)
// 	return (
// 		escaped
// 			// Kommentare einfärben
// 			.replace(/(&lt;!--[\s\S]*?--&gt;)/g, '<span class="xml-comment">$1</span>')
// 			// Tags INKLUSIVE Klammern einfärben:
// 			.replace(/(&lt;\/?)([\w:-]+)([^&]*?)(\/?&gt;)/g, (_match, open, tagName, attrs, close) => {
// 				const highlightedAttrs = attrs.replace(
// 					/([\w:-]+)=("[^"]*"|'[^']*')/g,
// 					'<span class="xml-attr">$1</span>=<span class="xml-string">$2</span>'
// 				);

// 				// Hier fügen wir open (&lt;) und close (&gt;) wieder mit in den Tag-Span ein!
// 				return `<span class="xml-tag">${open}${tagName}</span>${highlightedAttrs}<span class="xml-tag">${close}</span>`;
// 			})
// 	);
// }

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
