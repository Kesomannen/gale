<script lang="ts">
	export let value: string;
	export let mono: boolean = false;
</script>

<textarea
	bind:value
	on:change
	{...$$restProps}
	autocomplete="off"
	class="valid:focus:ring-accent-400 h-40 min-h-11 w-full rounded-lg border border-gray-500 border-opacity-0 bg-gray-900 px-3 py-2 text-gray-300 placeholder-gray-400 invalid:ring-red-500 hover:border-opacity-100 focus:border-opacity-0 focus:outline-none focus:ring-2 {mono
		? 'font-mono'
		: 'font-sans'}"
/>
