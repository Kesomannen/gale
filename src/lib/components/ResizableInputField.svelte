<script lang="ts">
	export let value: string;
	export let mono: boolean = false;
</script>

<textarea
	bind:value
	on:change
	{...$$restProps}
	autocomplete="off"
	class="h-40 min-h-11 w-full rounded-lg border border-slate-500 border-opacity-0 bg-slate-900 px-3 py-2 text-slate-300 placeholder-slate-400 invalid:ring-red-500 hover:border-opacity-100 focus:border-opacity-0 focus:outline-none focus:ring-2 valid:focus:ring-accent-400 {mono
		? 'font-mono'
		: 'font-sans'}"
/>
