<script lang="ts">
	type Props = {
		label: any;
		description: any;
		required?: boolean;
		children?: import('svelte').Snippet;
	};

	let { label, description, required = false, children }: Props = $props();
</script>

<div class="my-1 invalid:mx-5">
	<div class="mb-[-3px] pl-0.5">
		<span class="text-primary-100 text-lg font-medium">{label}</span>
		{#if required}
			<span class="text-red-400">*</span>
		{/if}
	</div>
	<p class="text-primary-300 pb-1 pl-0.5">{description}</p>

	{@render children?.()}
</div>
