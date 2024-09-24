import React, { useState, useEffect } from "react";
import { Button } from "./ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { TrashIcon, PlusIcon } from "lucide-react";

interface Shortcut {
  id?: number;
  keys: string;
}

interface AddShortcutFormProps {
  onClose: () => void;
  onSave: (shortcut: Shortcut) => void;
  existingShortcut: Shortcut | null;
  onDelete?: () => void; // Optional delete handler
  isOpen: boolean; // Track form visibility
  openForm?: () => void; // Handler to open the form
}

export  const displayKey = (key: string) => {
  switch (key) {
    case " ":
      return "Space";
    case "ArrowUp":
      return "Up";
    case "ArrowDown":
      return "Down";
    case "ArrowLeft":
      return "Left";
    case "ArrowRight":
      return "Right";
    default:
      return key.charAt(0).toUpperCase() + key.slice(1);
  }
};

const AddShortcutForm: React.FC<AddShortcutFormProps> = ({
  onSave,
  existingShortcut,
  onClose,
  onDelete,
  isOpen,
  openForm,
}) => {
  const [keys, setKeys] = useState(
    existingShortcut ? existingShortcut.keys : ""
  );
  const [isCapturing, setIsCapturing] = useState(false);

  useEffect(() => {
    if (isOpen) setIsCapturing(true);

    const pressedKeys = new Set<string>(); // Track pressed keys

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      const isMac = navigator.platform.toUpperCase().includes("MAC");

      // Track modifiers and keys
      if (event.metaKey && isMac) pressedKeys.add("Cmd");
      if (event.ctrlKey && !isMac) pressedKeys.add("Ctrl");
      if (event.shiftKey) pressedKeys.add("Shift");
      if (event.altKey) pressedKeys.add("Alt");

      const key = normalizeKey(event.key);
      if (!["Control", "Shift", "Alt", "Meta"].includes(key)) {
        pressedKeys.add(key);
      }

      // Combine the pressed keys
      setKeys(Array.from(pressedKeys).join("+"));
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      const key = normalizeKey(event.key);
      pressedKeys.delete(key); // Remove key from pressed keys on release
    };

    const handleClickOutside = () => setIsCapturing(false);

    if (isCapturing) {
      window.addEventListener("keydown", handleKeyDown);
      window.addEventListener("keyup", handleKeyUp);
      document.addEventListener("click", handleClickOutside);
    } else {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      document.removeEventListener("click", handleClickOutside);
    }

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      document.removeEventListener("click", handleClickOutside);
    };
  }, [isCapturing, isOpen]);

  const normalizeKey = (key: string) => {
    switch (key) {
      case " ":
        return "Space";
      case "ArrowUp":
        return "ArrowUp";
      case "ArrowDown":
        return "ArrowDown";
      case "ArrowLeft":
        return "ArrowLeft";
      case "ArrowRight":
        return "ArrowRight";
      default:
        return key;
    }
  };
 
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (keys) {
      const shortcut: Shortcut = existingShortcut
        ? { ...existingShortcut, keys }
        : { keys };
      onSave(shortcut);
      onClose(); // Close form
    }
  };

  return (
    <>
      <Dialog
        open={isOpen}
        onOpenChange={(open) => {
          if (!open) {
            onClose(); // Reset state on close
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {existingShortcut ? "Edit Shortcut" : "Add Shortcut"}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-4 py-4">
            <div className="p-4 bg-gray-100 h-[68px]">
              <div className="flex space-x-2 text-3xl text-center text-gray-400">
                {displayKey(keys)}
              </div>
            </div>

            <div className="flex flex-col md:flex-row justify-end gap-4">
              <Button className="w-full" variant="secondary" onClick={onClose}>
                Cancel
              </Button>
              <Button className="w-full" variant="default" type="submit">
                {existingShortcut ? "Update" : "Add"}
              </Button>
            </div>

            {existingShortcut && onDelete && (
              <div className="flex justify-end">
                <Button
                  variant="ghost"
                  onClick={onDelete}
                  className="flex items-center gap-2 w-full"
                >
                  <TrashIcon className="h-5 w-5" />
                  Delete
                </Button>
              </div>
            )}
          </form>
        </DialogContent>
      </Dialog>

      {/* Button to add a new shortcut */}
      {!existingShortcut && (
        <Button
          onClick={openForm}
          variant="outline"
          className="h-full rounded-xl border text-gray-500 p-4 flex items-center justify-center flex-col gap-2"
        >
          <PlusIcon className="h-6 w-6" />
          <p className="text-sm">Add shortcut</p>
        </Button>
      )}
    </>
  );
};

export default AddShortcutForm;
