<script lang="ts">
    export let value: string;
</script>

<textarea
	class="w-full h-32 min-h-11 px-3 py-2 rounded-lg bg-gray-900 placeholder-slate-400
    text-slate-300
    valid:focus:ring-green-400 invalid:ring-red-500 focus:ring-2 focus:outline-none
    border border-slate-500 border-opacity-0 hover:border-opacity-100 focus:border-opacity-0"
	bind:value
    {...$$restProps}
    on:change
/>
