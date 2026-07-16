// Custom asset logo overrides - web adapter.
// The image is served directly by the backend at a stable URL, so no
// client-side caching/decoding step is needed the way Tauri requires.

const logoPath = (assetId: string): string => `/api/v1/assets/${encodeURIComponent(assetId)}/logo`;

export const getAssetLogoUrl = async (
  asset: { id: string; customLogoFilename?: string | null } | undefined,
): Promise<string | null> => {
  if (!asset?.customLogoFilename) return null;
  return logoPath(asset.id);
};

export const uploadAssetLogo = async (assetId: string, file?: File): Promise<boolean> => {
  if (!file) throw new Error("A file is required to upload a logo");
  const formData = new FormData();
  formData.append("file", file);
  const res = await fetch(logoPath(assetId), {
    method: "POST",
    body: formData,
    credentials: "same-origin",
  });
  if (!res.ok) {
    throw new Error(`Failed to upload logo (${res.status})`);
  }
  return true;
};

export const removeAssetLogo = async (assetId: string): Promise<void> => {
  const res = await fetch(logoPath(assetId), {
    method: "DELETE",
    credentials: "same-origin",
  });
  if (!res.ok) {
    throw new Error(`Failed to remove logo (${res.status})`);
  }
};
