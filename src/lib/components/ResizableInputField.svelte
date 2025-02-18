<script lang="ts">
	export let value: string;
	export let mono: boolean = false;
</script>

<textarea
	bind:value
	on:change
	{...$$restProps}
	autocomplete="off"
	class="valid:focus:ring-accent-400 h-40 min-h-11 w-full rounded-lg border border-transparent bg-slate-900 px-3 py-2 text-slate-300 placeholder-slate-400 invalid:ring-red-500 hover:border-slate-500 focus:border-transparent focus:ring-2 focus:outline-hidden {mono
		? 'font-mono'
		: 'font-sans'}"
/>
