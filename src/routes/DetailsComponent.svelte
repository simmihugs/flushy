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
	<pre class="xml-container"><code
			>{#await highlightXml(event.xmlString)}
				Formatting...
			{:then highlighted}
				{@html highlighted}
			{:catch error}
				Formatierungsfehler
			{/await}</code
		></pre>
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
		background-color: var(--xml-container-background);
		padding: 12px;
		border-radius: 6px;
		font-family: 'Courier New', Courier, monospace;
		font-size: 10px !important;
		line-height: 1.2;
		margin: 0;
		text-align: left;
		box-sizing: border-box;

		white-space: pre;
		overflow-x: auto;
	}

	:global(.xml-bracket) {
		color: var(--xml-bracket);
		font-weight: normal;
	}

	:global(.xml-tag) {
		color: var(--xml-tag);
		font-weight: bold;
	}

	:global(.xml-attr) {
		color: var(--xml-attr);
	}

	:global(.xml-string) {
		color: var(--xml-string);
	}

	:global(.xml-comment) {
		color: #64748b;
		font-style: italic;
	}
</style>
