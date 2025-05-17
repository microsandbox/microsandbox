using System;

namespace Microsandbox
{
    /// <summary>
    /// Core class for the Microsandbox SDK.
    /// </summary>
    public class Microsandbox
    {
        /// <summary>
        /// Gets the version of the Microsandbox SDK.
        /// </summary>
        /// <returns>The version of the SDK</returns>
        public static string GetVersion()
        {
            // This should be updated whenever the SDK version changes
            return "0.1.0";
        }

        /// <summary>
        /// Initializes a new instance of the Microsandbox SDK.
        /// </summary>
        public Microsandbox()
        {
            // Initialization logic will go here
            Console.WriteLine("Microsandbox SDK initialized");
        }
    }
}
