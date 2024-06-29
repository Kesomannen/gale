<script lang="ts">
	export let label;
	export let description;
	export let required = false;
</script>

<div class="my-1 invalid:mx-5">
	<div class="pl-0.5 mb-[-3px]">
		<span class="text-lg text-slate-100 font-medium">{label}</span>
		{#if required}
			<span class="text-red-400">*</span>
		{/if}
	</div>
	<p class="text-slate-300 pl-0.5 pb-1">{description}</p>

	<slot />
</div>
