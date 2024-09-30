import React, { useState, useEffect, useRef, useCallback } from "react";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "./ui/dialog";
import { TrashIcon, PlusIcon, Circle, Square } from "lucide-react";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import Prism from "prismjs";
import "prismjs/components/prism-json";
import "prismjs/themes/prism.css";
import clsx from "clsx";

export interface Shortcut {
  id?: number;
  name?: string;
  sequence: string[];
}

interface AddShortcutFormProps {
  onClose: () => void;
  onSave: (shortcut: Shortcut) => Promise<void>;
  existingShortcut: Shortcut | null;
  onDelete?: () => void;
  isOpen: boolean;
  openForm?: () => void;
}

export const displayKey = (key: string) => {
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
      return key;
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
  const [name, setName] = useState(
    existingShortcut ? existingShortcut.name : ""
  );
  const [sequence, setSequence] = useState<string[]>(
    existingShortcut ? existingShortcut.sequence : []
  );
  const [isCapturing, setIsCapturing] = useState(false);
  const [activeTab, setActiveTab] = useState("shortcut");

  // State for JSON Editor
  const [jsonInput, setJsonInput] = useState(
    existingShortcut ? JSON.stringify(existingShortcut, null, 2) : ""
  );
  const [jsonError, setJsonError] = useState("");

  // State for recording in Record tab
  const [isRecording, setIsRecording] = useState(false);

  // Ref for debounce timeout
  const debounceTimeout = useRef<number | null>(null);

  useEffect(() => {
    if (isOpen && activeTab === "shortcut") setIsCapturing(true);

    const modifierKeys = new Set<string>(); // Track pressed modifier keys
    const regularKeys = new Set<string>(); // Track pressed regular keys

    // Define which keys are considered modifiers
    const isModifierKey = (key: string) => {
      return (
        key === "Control" || key === "Shift" || key === "Alt" || key === "Meta"
      );
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      // Prevent default behavior to avoid unintended actions
      event.preventDefault();

      const key = event.key;

      // Avoid capturing repeated keydown events for the same key
      if (event.repeat) return;

      // Determine if the key is a modifier
      if (isModifierKey(key)) {
        modifierKeys.add(key);
      } else {
        regularKeys.add(key);
      }

      // Form the key combination
      const modifiers = Array.from(modifierKeys).map((mod) => displayKey(mod));
      const regulars = Array.from(regularKeys).map((reg) => displayKey(reg));

      let keyCombination = "";
      if (modifiers.length > 0 && regulars.length > 0) {
        keyCombination = modifiers.join("+") + "+" + regulars.join("+");
      } else if (regulars.length > 0) {
        keyCombination = regulars.join("+");
      } else {
        // If only modifiers are pressed, do not capture
        return;
      }

      // Implement debouncing: clear existing timeout
      if (debounceTimeout.current) {
        clearTimeout(debounceTimeout.current);
      }

      // Set a new timeout to record the key combination after 200ms
      debounceTimeout.current = window.setTimeout(() => {
        if (activeTab === "shortcut") {
          setSequence([keyCombination]);
          setName(keyCombination); // For Shortcut tab, name is same as keys
          // Stop capturing after the first shortcut
          setIsCapturing(false);
        } else if (activeTab === "record" && isRecording) {
          // For Record tab, add to sequence
          setSequence((prevSequence) => [...prevSequence, keyCombination]);
        }
      }, 200); // 200ms debounce delay
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      const key = event.key;

      // Remove key from respective sets
      if (isModifierKey(key)) {
        modifierKeys.delete(key);
      } else {
        regularKeys.delete(key);
      }
    };

    const handleClickOutside = () => setIsCapturing(false);

    if (
      (isCapturing && activeTab === "shortcut") ||
      (isRecording && activeTab === "record")
    ) {
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
      if (debounceTimeout.current) {
        clearTimeout(debounceTimeout.current);
      }
    };
  }, [isCapturing, isRecording, isOpen, activeTab]);

  useEffect(() => {
    if (activeTab === "json") {
      // Highlight the JSON syntax
      Prism.highlightAll();
    }
  }, [jsonInput, activeTab]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    let shortcut: Shortcut | null = null;

    if (activeTab === "shortcut") {
      if (sequence.length > 0) {
        shortcut = existingShortcut
          ? { ...existingShortcut, sequence, name: name || sequence[0] }
          : { sequence, name: name || sequence[0] };
      }
    } else if (activeTab === "record") {
      if (sequence.length > 0 && name) {
        shortcut = existingShortcut
          ? { ...existingShortcut, sequence, name }
          : { sequence, name };
      }
    } else if (activeTab === "json") {
      try {
        const parsedJson = JSON.parse(jsonInput);
        if (parsedJson.sequence && parsedJson.name) {
          shortcut = existingShortcut
            ? { ...existingShortcut, ...parsedJson }
            : parsedJson;
        } else {
          setJsonError("JSON must contain 'sequence' and 'name' fields.");
          return;
        }
      } catch (error) {
        setJsonError("Invalid JSON format.");
        return;
      }
    }

    if (shortcut) {
      onSave(shortcut);
      onClose(); // Close form
    }
  };

  const handleJsonChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setJsonInput(e.target.value);
    setJsonError(""); // Reset error on change
  };

  const handleStartRecording = () => {
    setIsRecording(true);
    setSequence([]); // Reset sequence
  };

  const handleStopRecording = () => {
    setIsRecording(false);
  };

  // Function to delete a specific key combination from the sequence
  const handleDeleteCombination = (index: number) => {
    setSequence((prevSequence) => prevSequence.filter((_, i) => i !== index));
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
        <DialogContent className="overflow-y-auto max-h-full">
          <DialogHeader>
            <DialogTitle>
              {existingShortcut ? "Edit Shortcut" : "Add Shortcut"}
            </DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit} className="space-y-4 py-4">
            <Tabs
              defaultValue="shortcut"
              className="w-full"
              onValueChange={(value) => {
                setActiveTab(value);
                setIsCapturing(value === "shortcut");
                setIsRecording(false); // Stop recording when changing tabs
              }}
            >
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="shortcut">Shortcut</TabsTrigger>
                <TabsTrigger value="record">Record</TabsTrigger>
                <TabsTrigger value="json">JSON Editor</TabsTrigger>
              </TabsList>
              <TabsContent value="shortcut">
                <div className="space-y-4">
                  <div className="p-4 bg-muted rounded h-[68px] flex items-center justify-center">
                    {sequence.length > 0 ? (
                      sequence.map((combo, idx) => (
                        <div className="block capitalize truncate max-w-[calc(100vw-100px)] space-x-2 text-3xl text-center text-gray-400">
                          {combo}
                        </div>
                      ))
                    ) : (
                      <div className="text-gray-400">
                        Press a key combination
                      </div>
                    )}
                  </div>
                </div>
              </TabsContent>
              <TabsContent value="record">
                <div className="space-y-4">
                  <div className="flex gap-4">
                    <div className="w-full">
                      <Label htmlFor="name">Name</Label>
                      <Input
                        id="name"
                        value={name}
                        onChange={(e) => setName(e.target.value)}
                        placeholder="Enter shortcut name"
                      />
                    </div>
                    <div className="flex items-end gap-2">
                        <Button
                          variant={isRecording ? "destructive" : "outline"}
                          onClick={
                            isRecording
                              ? handleStopRecording
                              : handleStartRecording
                          }
                        >
                          <span className="mr-2">
                            {isRecording ? (
                              <Square size={14} />
                            ) : (
                              <Circle size={14} />
                            )}
                          </span>
                          {isRecording ? "Stop Recording" : "Start Recording"}
                        </Button>
                    </div>
                  </div>

                  <div className="p-4 bg-muted rounded">
                    
                    <div className="flex flex-col space-y-2 text-center text-gray-700">
                      {sequence.length > 0 ? (
                        sequence.map((keyCombo, index) => (
                          <div
                            key={index}
                            className="flex items-center justify-between space-x-2"
                          >
                            <span className="capitalize">{keyCombo}</span>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => handleDeleteCombination(index)}
                              aria-label={`Delete ${keyCombo}`}
                            >
                              <TrashIcon className="h-4 w-4 text-red-500" />
                            </Button>
                          </div>
                        ))
                      ) : (
                        <div className="text-gray-400">
                          No key combinations recorded
                        </div>
                      )}
                    </div>
                  </div>
                </div>
              </TabsContent>
              <TabsContent value="json">
                <div className="space-y-4">
                  <div>
                    <Label htmlFor="json">Shortcut JSON</Label>
                    <textarea
                      id="json"
                      value={jsonInput}
                      onChange={handleJsonChange}
                      className={clsx(
                        "w-full h-40 p-2 border rounded-md font-mono text-sm",
                        jsonError ? "border-red-500" : "border-gray-300"
                      )}
                    />
                    {jsonError && (
                      <p className="text-red-500 text-sm language-json">
                        {jsonError}
                      </p>
                    )}
                  </div>
                </div>
              </TabsContent>
            </Tabs>

            <div className="flex flex-col md:flex-row justify-end gap-4">
              <Button
                className="w-full order-2 md:order-1"
                variant="secondary"
                onClick={onClose}
              >
                Cancel
              </Button>
              <Button
                className="w-full order-1 md:order-2"
                variant="default"
                type="submit"
              >
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
      {!existingShortcut && openForm && (
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
