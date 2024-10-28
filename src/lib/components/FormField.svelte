<script lang="ts">
	export let label;
	export let description;
	export let required = false;
</script>

<div class="my-1 invalid:mx-5">
	<div class="mb-[-3px] pl-0.5">
		<span class="text-lg font-medium text-gray-100">{label}</span>
		{#if required}
			<span class="text-red-400">*</span>
		{/if}
	</div>
	<p class="pb-1 pl-0.5 text-gray-300">{description}</p>

	<slot />
</div>
