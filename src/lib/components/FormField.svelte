<script lang="ts">
	export let label;
	export let description;
	export let required = false;
</script>

<div class="my-1 invalid:mx-5">
	<div class="mb-[-3px] pl-0.5">
		<span class="text-primary-100 text-lg font-medium">{label}</span>
		{#if required}
			<span class="text-red-400">*</span>
		{/if}
	</div>
	<p class="text-primary-300 pb-1 pl-0.5">{description}</p>

	<slot />
</div>
