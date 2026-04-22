import { fireEvent, render, screen } from "@testing-library/react";
import { type UpdateNotificationChannelInput } from "@ipotto/shared";
import { describe, expect, it, vi } from "vitest";

import { NotificationChannelForm } from "./NotificationChannelForm";

const baseValue: UpdateNotificationChannelInput = {
  channelType: "Email",
  destination: { address: "alert@example.com" },
  enabled: true,
  subscriptions: { ApplicationCompleted: true },
};

describe("NotificationChannelForm", () => {
  it("shows the destination field matching the channel type", () => {
    render(
      <NotificationChannelForm
        value={baseValue}
        onChange={() => undefined}
        onRemove={() => undefined}
      />,
    );
    expect(screen.getByDisplayValue("alert@example.com")).toBeDefined();
  });

  it("emits onChange when the enabled toggle flips", () => {
    const onChange = vi.fn();
    render(
      <NotificationChannelForm
        value={baseValue}
        onChange={onChange}
        onRemove={() => undefined}
      />,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "有効" }));
    expect(onChange).toHaveBeenCalledWith({ ...baseValue, enabled: false });
  });

  it("invokes onRemove when the delete button is clicked", () => {
    const onRemove = vi.fn();
    render(
      <NotificationChannelForm
        value={baseValue}
        onChange={() => undefined}
        onRemove={onRemove}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "削除" }));
    expect(onRemove).toHaveBeenCalledTimes(1);
  });
});
