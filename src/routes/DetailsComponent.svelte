<script lang="ts">
	import Prism from 'prismjs';
	import 'prismjs/components/prism-markup';
	import 'prismjs/themes/prism-tomorrow.css';
	import { formatTime } from './helper';
	let { event } = $props();

	//let highlightedCode = $derived.by(() => {
	//	console.log(event.xmlString);
	//	if (!event || !event.xmlSource) {
	//		return '';
	//	}
	//	try {
	//		return Prism.highlight(event.xmlString, Prism.languages.xml, 'xml');
	//	} catch (e) {
	//		console.error('Highlighting fehlgeschlagen', e);
	//		return event.xmlString;
	//	}
	//});
</script>

<div class="details-card">
	<h2>Details</h2>

	<div class="meta-info">
		<p><strong>ID:</strong> {event.eventId}</p>
		<p><strong>Titel:</strong> {event.title}</p>
		<p><strong>Start:</strong> {formatTime(event.startTime)}</p>
		<p><strong>Ende:</strong> {formatTime(event.endTime)}</p>
	</div>

	<h3>XML Source</h3>
	<!-- <pre class="language-xml"> -->
	<!--   <code class="language-xml" -->
	<!-- 		>{@html highlightedCode} -->
	<!-- 	</code> -->
	<!-- </pre> -->
	<pre class="language-xml xml-container">
    <code class="language-xml">{event.xmlString}</code>
</pre>
</div>

<style>
	.details-card {
		padding: 5px;
		color: #e2e8f0;
		height: 100%;
		overflow-y: auto;
	}

	.meta-info {
		margin-bottom: 20px;
		font-size: 0.9em;
		line-height: 1.6;
	}

	.xml-container {
		background: #1e293b;
		padding: 5px;
		border-radius: 5px;
		overflow-x: auto;
		font-family: 'Courier New', Courier, monospace;
		font-size: 0.1em;
	}
	.xml-container {
		background: #1e293b;
		padding: 10px;
		border-radius: 5px;
		overflow-x: auto;
		font-family: 'Courier New', Courier, monospace;
		/* !important erzwingt die Größe gegen Browser-Defaults */
		font-size: 10px !important;
		line-height: 1.2;
	}

	pre {
		margin: 0;
		white-space: pre-wrap;
		word-break: break-all;
	}

	pre {
		margin: 0;
		white-space: pre-wrap;
		word-break: break-all;
	}

	.error {
		color: #f87171;
	}
</style>
