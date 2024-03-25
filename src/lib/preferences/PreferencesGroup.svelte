<script lang="ts">
	export let title: string;
</script>

<div class="my-3 ml-1">
	<div class="text-slate-100 text-xl font-medium pb-1">
		{title}
	</div>

	<div class="flex flex-col gap-1 ml-2">
		<slot />
	</div>
</div>
