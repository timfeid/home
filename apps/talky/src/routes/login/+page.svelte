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
    password: '',
  });

  async function handleSubmit(event: Event) {
    event.preventDefault();
    try {
      await user.login(formData);
      return goto('/');
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div class="container mx-auto flex min-h-[80vh] items-center justify-center px-4 py-8">
  <Card class="w-full max-w-md border-0 bg-transparent">
    <CardHeader>
      <CardTitle class="text-2xl">Login</CardTitle>
      <CardDescription>Enter your credentials to access your account</CardDescription>
    </CardHeader>
    <form onsubmit={handleSubmit}>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <Label for="username">Username</Label>
          <Input
            id="username"
            name="username"
            autofocus
            type="text"
            placeholder="Your username"
            required
            bind:value={formData.username}
          />
        </div>
        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <Label for="password">Password</Label>
            <Button
              variant="link"
              tabindex="1"
              href="/forgot-password"
              class="text-sm text-primary/50 hover:text-primary">Forgot password?</Button
            >
          </div>
          <Input
            id="password"
            name="password"
            type="password"
            placeholder="Your password"
            required
            bind:value={formData.password}
          />
        </div>
      </CardContent>
      <CardFooter class="flex flex-col space-y-4">
        <Button type="submit" class="w-full">Login</Button>
        <div class="text-center text-sm">
          Don&apos;t have an account?
          <Button variant="link" href="/signup" class="px-2">Sign up</Button>
        </div>
      </CardFooter>
    </form>
  </Card>
</div>
