import { test, expect } from '@playwright/test';
import { mockApi, workspace, workspaceId } from './fixtures';
import { installMockWebSocket } from './websocket-mock';

test.beforeEach(async ({ page }) => {
  await mockApi(page);
  await installMockWebSocket(page);
  await page.goto('/');
  await page.waitForResponse('**/api/bootstrap');
  await page.getByText(workspace.name, { exact: true }).first().hover();
});

async function openRename(page: import('@playwright/test').Page) {
  await page
    .getByRole('button', { name: `Rename workspace ${workspace.name}`, exact: true })
    .click();
  await expect(page.getByRole('dialog', { name: 'Rename workspace' })).toBeVisible();
}

test('modal shows the current name and Cancel closes it', async ({ page }) => {
  await openRename(page);
  await expect(page.getByLabel('Workspace name')).toHaveValue(workspace.name);
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByRole('dialog')).toHaveCount(0);
});

test('Escape cancels rename', async ({ page }) => {
  await openRename(page);
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toHaveCount(0);
});

test('Enter renames and immediately updates the selected sidebar workspace', async ({ page }) => {
  let requestName: string | undefined;
  await page.route(`**/api/workspaces/${workspaceId}/rename`, async (route) => {
    requestName = (route.request().postDataJSON() as { name: string }).name;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...workspace, name: requestName.trim() }),
    });
  });
  await openRename(page);
  await page.getByLabel('Workspace name').fill('  Render Engine  ');
  await page.getByLabel('Workspace name').press('Enter');
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await expect(page.getByText('Render Engine', { exact: true })).toBeVisible();
  expect(requestName).toBe('  Render Engine  ');
  await expect(page.getByRole('button', { name: /main Render Engine Rename/ })).toHaveClass(
    /bg-accent\/10/
  );
});

test('shows loading while saving and a friendly duplicate error', async ({ page }) => {
  await page.route(`**/api/workspaces/${workspaceId}/rename`, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 250));
    await route.fulfill({
      status: 409,
      contentType: 'application/json',
      body: JSON.stringify({ error: 'Conflict', details: 'duplicate' }),
    });
  });
  await openRename(page);
  await page.getByLabel('Workspace name').fill('Duplicate');
  await page.getByRole('button', { name: 'Rename', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Renaming…' })).toBeDisabled();
  await expect(page.getByRole('alert')).toHaveText(
    'A workspace with this name already exists in this repository.'
  );
  await expect(page.getByRole('dialog')).toBeVisible();
});
