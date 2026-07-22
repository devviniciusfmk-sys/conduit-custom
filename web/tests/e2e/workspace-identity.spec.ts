import { test, expect, type Page } from '@playwright/test';
import { mockApi, workspace, workspaceId } from './fixtures';
import { installMockWebSocket } from './websocket-mock';

test.beforeEach(async ({ page }) => {
  await mockApi(page);
  await installMockWebSocket(page);
  await page.goto('/');
  await page.waitForResponse('**/api/bootstrap');
});

async function openSettings(page: Page) {
  await page.getByText(workspace.name, { exact: true }).first().hover();
  await page
    .getByRole('button', { name: `Workspace settings ${workspace.name}`, exact: true })
    .click();
  await expect(page.getByRole('dialog', { name: 'Workspace settings' })).toBeVisible();
}

test('renders workspace icon and opens identity editor', async ({ page }) => {
  await expect(page.getByText('📁', { exact: true })).toHaveClass(/text-gray-400/);
  await expect(page.getByText('Live Jade', { exact: true }).first()).toBeVisible();
  await openSettings(page);
  await expect(page.getByLabel('Name')).toHaveValue(workspace.name);
  await expect(page.getByRole('button', { name: 'Icon 📁' })).toHaveAttribute('aria-pressed', 'true');
  await expect(page.getByRole('button', { name: 'Color gray' })).toHaveAttribute('aria-pressed', 'true');
});

test('Cancel discards identity changes', async ({ page }) => {
  await openSettings(page);
  await page.getByLabel('Name').fill('Discarded');
  await page.getByRole('button', { name: 'Icon 🎬' }).click();
  await page.getByRole('button', { name: 'Color purple' }).click();
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.getByText(workspace.name, { exact: true }).first()).toBeVisible();
});

test('persists name, icon, and color and updates the sidebar', async ({ page }) => {
  let payload: unknown;
  await page.route(`**/api/workspaces/${workspaceId}/identity`, async (route) => {
    payload = route.request().postDataJSON();
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...workspace, ...(payload as object), name: 'Render Engine' }),
    });
  });
  await openSettings(page);
  await page.getByLabel('Name').fill('Render Engine');
  await page.getByRole('button', { name: 'Icon 🎬' }).click();
  await page.getByRole('button', { name: 'Color purple' }).click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();

  expect(payload).toEqual({ name: 'Render Engine', icon: '🎬', color: 'purple' });
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.getByText('🎬', { exact: true })).toHaveClass(/text-purple-400/);
  await expect(page.getByText('Render Engine', { exact: true })).toBeVisible();
});

test('shows loading while identity is being saved', async ({ page }) => {
  await page.route(`**/api/workspaces/${workspaceId}/identity`, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 300));
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...workspace, icon: '🚀', color: 'orange' }),
    });
  });
  await openSettings(page);
  await page.getByRole('button', { name: 'Icon 🚀' }).click();
  await page.getByRole('button', { name: 'Color orange' }).click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Saving…' })).toBeDisabled();
  await expect(page.getByRole('dialog')).toHaveCount(0);
});
