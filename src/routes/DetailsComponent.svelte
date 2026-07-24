<script lang="ts">
	import { formatTime, highlightXml } from './helper';
	let { event } = $props();
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
	<pre class="xml-container"><code>{@html highlightXml(event.xmlString)}</code></pre>
</div>

<style>
	.details-card {
		padding: 5px;
		color: var(--details-card-color);
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

	.xml-container {
		background: #1e293b;
		padding: 12px;
		border-radius: 6px;
		font-family: 'Courier New', Courier, monospace;
		font-size: 11px;
		line-height: 1.4;
		white-space: pre;
		overflow-x: auto;
		text-align: left;
		margin: 0;
	}

	:global(.xml-tag) {
		color: #f43f5e;
		font-weight: bold;
	}

	:global(.xml-attr) {
		color: #38bdf8;
	}

	:global(.xml-string) {
		color: #facc15;
	}

	:global(.xml-comment) {
		color: #64748b;
		font-style: italic;
	}
</style>
