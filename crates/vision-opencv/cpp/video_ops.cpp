#include "bridge_common.hpp"

static std::string op_video_frame(const std::vector<std::string>& args) {
    if (args.size() < 4) fail("video_frame requires path, frame_index, output");
    const std::string& path = args[1];
    const int frame_index = std::stoi(args[2]);
    const std::string& output = args[3];
    cv::VideoCapture capture(path);
    if (!capture.isOpened()) fail("failed to open video: " + path);
    if (frame_index < 0 || !capture.set(cv::CAP_PROP_POS_FRAMES, frame_index))
        fail("invalid video frame index: " + std::to_string(frame_index));
    cv::Mat frame;
    if (!capture.read(frame) || frame.empty())
        fail("failed to read video frame " + std::to_string(frame_index) + ": " + path);
    write_image(output, frame);
    return metadata_json(frame, output);
}
