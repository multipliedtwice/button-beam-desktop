import React, { useState } from "react";
import { invoke } from "@tauri-apps/api";
import { QrCode } from "lucide-react";
import { Button } from "./ui/button";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "./ui/dialog";
import QRCode from "react-qr-code";

const ConnectWithQR: React.FC = () => {
  const [isQrOpen, setIsQrOpen] = useState(false);
  const [qrData, setQrData] = useState<string | null>(null);

  const fetchLocalIp = async () => {
    try {
      // Fetch the local IP address and a free port
      const ip = await invoke<string>("get_local_ip");
      const port = await invoke<number>("find_free_port");
      setQrData(`${ip}:${port}`);  // Combine IP and port
      setIsQrOpen(true);           // Open the dialog
    } catch (error) {
      console.error("Error fetching local IP or port:", error);
    }
  };

  return (
    <div>
      {/* Button to open QR dialog */}
      <Button variant="ghost" onClick={fetchLocalIp}>
        <QrCode className="h-4 w-4 mr-1" />
        Connect with QR
      </Button>

      {/* QR Code Dialog */}
      <Dialog open={isQrOpen} onOpenChange={setIsQrOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Connect with QR</DialogTitle>
          </DialogHeader>
          <div className="flex flex-col justify-center items-center gap-8">
            {qrData ? (
              <>
                <div className="flex flex-col justify-center items-center gap-4">
                    <p className="text-sm text-gray-500">Scan with Button Beam mobile app</p>
                    <QRCode value={qrData} />
                    {/* Display IP and Port below the QR code */}
                    <p className="text-sm text-gray-500">Or enter manually: {' '}
                        <span className="bg-gray-100 font-mono p-1 px-2 text-gray-700">
                        {qrData}
                        </span>
                    </p>
                </div>
              </>
            ) : (
              <p>Loading QR Code...</p>
            )}
            <Button onClick={() => setIsQrOpen(false)}>Close</Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default ConnectWithQR;
