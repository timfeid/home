<script lang="ts">
  import { goto } from '$app/navigation';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { user } from '$lib/user.svelte';

  const formData = $state({
    username: '',
    email: '',
    password: '',
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();
    try {
      await user.register(formData);
      return goto('/');
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div class="container mx-auto flex min-h-[80vh] items-center justify-center px-4 py-8">
  <Card class="w-full max-w-md border-0 bg-transparent">
    <CardHeader>
      <CardTitle class="text-2xl">Sign Up</CardTitle>
      <CardDescription>Create an account to get started</CardDescription>
    </CardHeader>
    <form on:submit={handleSubmit}>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Label for="username">Username</Label>
          <Input
            id="username"
            name="username"
            type="text"
            placeholder="Your username"
            required
            bind:value={formData.username}
          />
        </div>
        <div class="space-y-2">
          <Label for="email">Email</Label>
          <Input
            id="email"
            name="email"
            type="email"
            placeholder="you@example.com"
            required
            bind:value={formData.email}
          />
        </div>
        <div class="space-y-2">
          <Label for="password">Password</Label>
          <Input
            id="password"
            name="password"
            type="password"
            placeholder="Create a password"
            required
            bind:value={formData.password}
          />
        </div>
      </CardContent>
      <CardFooter class="flex flex-col space-y-4">
        <Button type="submit" class="w-full">Sign Up</Button>
        <div class="text-center text-sm">
          Already have an account?
          <Button variant="link" href="/login" class="px-2">Log in</Button>
        </div>
      </CardFooter>
    </form>
  </Card>
</div>
