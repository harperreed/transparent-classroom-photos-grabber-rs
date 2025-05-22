import requests
import os
import json
from datetime import datetime
import subprocess
from bs4 import BeautifulSoup
from fractions import Fraction
from dotenv import load_dotenv
import piexif
import logging
from typing import Optional, List, Dict, Any

class TransparentClassroomClient:
    def __init__(self, email: str, password: str, school_id: int, child_id: int,
                 school_lat: float = 0.0, school_lng: float = 0.0,
                 school_keywords: str = '', cache_dir: str = "./cache",
                 photo_dir: str = "./photos", cache_timeout: int = 14400):
        """
        Initialize the Transparent Classroom client

        Args:
            email: Login email
            password: Login password
            school_id: School ID
            child_id: Child ID
            school_lat: School latitude for photo metadata
            school_lng: School longitude for photo metadata
            school_keywords: Keywords for photo metadata
            cache_dir: Directory for caching API responses
            photo_dir: Directory for storing downloaded photos
            cache_timeout: Cache timeout in seconds (default 4 hours)
        """
        # Initialize logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger('TransparentClassroom')

        # Set instance variables
        self.session = requests.Session()
        self.school_id = school_id
        self.child_id = child_id
        self.school_lat = school_lat
        self.school_lng = school_lng
        self.school_keywords = school_keywords
        self.cache_dir = cache_dir
        self.photo_dir = photo_dir
        self.cache_timeout = cache_timeout

        # Create necessary directories
        os.makedirs(cache_dir, exist_ok=True)
        os.makedirs(photo_dir, exist_ok=True)

        # Initialize session
        self._login(email, password)

    def _login(self, email: str, password: str) -> bool:
        """Login to Transparent Classroom"""
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'en-US,en;q=0.5',
            'Content-Type': 'application/x-www-form-urlencoded',
        }

        try:
            # Get CSRF token
            login_url = 'https://www.transparentclassroom.com/souls/sign_in'
            response = self.session.get(login_url, headers=headers)
            response.raise_for_status()

            soup = BeautifulSoup(response.text, 'html.parser')
            csrf_token = soup.find('meta', {'name': 'csrf-token'})

            if not csrf_token:
                raise ValueError("Could not find CSRF token")

            login_data = {
                'authenticity_token': csrf_token['content'],
                'soul[login]': email,
                'soul[password]': password,
                'soul[remember_me]': '0',
                'commit': 'Sign in'
            }

            # Perform login
            response = self.session.post(login_url, data=login_data, headers=headers)
            response.raise_for_status()

            if 'You need to sign in' in response.text:
                raise ValueError("Invalid credentials")

            self.logger.info("Login successful")
            return True

        except (requests.exceptions.RequestException, ValueError) as e:
            self.logger.error(f"Login failed: {str(e)}")
            raise

    def _get_cached_data(self, cache_file: str) -> Optional[Dict]:
        """Get data from cache if valid"""
        if os.path.exists(cache_file):
            cache_age = datetime.now() - datetime.fromtimestamp(os.path.getmtime(cache_file))
            if cache_age.total_seconds() <= self.cache_timeout:
                self.logger.info(f"Loading cached data from {cache_file}")
                with open(cache_file, 'r') as file:
                    return json.load(file)
            else:
                self.logger.info(f"Cache expired, removing {cache_file}")
                os.remove(cache_file)
        return None

    def get_posts(self, page: int = 1) -> Optional[List[Dict[str, Any]]]:
        """Get posts for the specified page"""
        cache_file = f"{self.cache_dir}/cache_page_{page}.json"

        # Try cache first
        cached_data = self._get_cached_data(cache_file)
        if cached_data:
            return cached_data

        # Make API request
        url = f"https://www.transparentclassroom.com/s/{self.school_id}/children/{self.child_id}/posts.json"

        try:
            response = self.session.get(url, params={"locale": "en", "page": page})
            response.raise_for_status()

            # Cache response
            with open(cache_file, 'w') as file:
                json.dump(response.json(), file, indent=4, sort_keys=True)

            return response.json()

        except requests.exceptions.RequestException as e:
            self.logger.error(f"Failed to get posts: {str(e)}")
            return None

    @staticmethod
    def _to_deg(value: float, loc: List[str]) -> tuple:
        """Convert decimal coordinates to degrees, minutes, seconds"""
        loc_value = loc[1] if value < 0 else loc[0]
        abs_value = abs(value)
        deg = int(abs_value)
        t1 = (abs_value - deg) * 60
        min = int(t1)
        sec = round((t1 - min) * 60, 5)
        return deg, min, sec, loc_value

    @staticmethod
    def _change_to_rational(number: float) -> tuple:
        """Convert number to rational for EXIF"""
        f = Fraction(str(number))
        return (f.numerator, f.denominator)

    def set_iptc_metadata(self, image_path: str, title: str, creator: str) -> None:
        """Set IPTC metadata using exiftool"""
        try:
            command = [
                'exiftool',
                f'-IPTC:ObjectName={title}',
                f'-IPTC:By-line={creator}',
                f'-IPTC:Keywords={self.school_keywords}',
                image_path
            ]
            subprocess.run(command, check=True)
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Failed to set IPTC metadata: {str(e)}")
            raise

    def download_and_embed_metadata(self, photo_data: Dict[str, Any]) -> None:
        """Download photo and embed metadata"""
        try:
            photo_url = photo_data['original_photo_url']
            description = BeautifulSoup(photo_data['html'], 'html.parser').get_text()
            creator = BeautifulSoup(photo_data['author'], 'html.parser').get_text()
            created_at = datetime.fromisoformat(photo_data['created_at'].rstrip("Z"))
            photo_id = photo_data['id']

            image_path = f"{self.photo_dir}/{photo_id}_max.jpg"

            # Download if doesn't exist
            if not os.path.exists(image_path):
                response = self.session.get(photo_url)
                response.raise_for_status()

                with open(image_path, 'wb') as file:
                    file.write(response.content)

            # Update EXIF metadata
            exif_dict = piexif.load(image_path)

            # Basic EXIF data
            exif_dict['0th'][piexif.ImageIFD.ImageDescription] = description.encode('utf-8')
            exif_dict['Exif'][piexif.ExifIFD.DateTimeOriginal] = created_at.strftime("%Y:%m:%d %H:%M:%S").encode('utf-8')

            # GPS data
            lat_deg = self._to_deg(self.school_lat, ["N", "S"])
            lng_deg = self._to_deg(self.school_lng, ["E", "W"])

            exif_dict["GPS"] = {
                piexif.GPSIFD.GPSLatitudeRef: lat_deg[3].encode('utf-8'),
                piexif.GPSIFD.GPSLatitude: tuple(self._change_to_rational(x) for x in lat_deg[:3]),
                piexif.GPSIFD.GPSLongitudeRef: lng_deg[3].encode('utf-8'),
                piexif.GPSIFD.GPSLongitude: tuple(self._change_to_rational(x) for x in lng_deg[:3]),
            }

            # Write EXIF data
            exif_bytes = piexif.dump(exif_dict)
            piexif.insert(exif_bytes, image_path)

            # Set IPTC metadata
            self.set_iptc_metadata(image_path, description, creator)

            # Set file timestamps
            os.utime(image_path, (created_at.timestamp(), created_at.timestamp()))

            self.logger.info(f"Successfully processed photo {photo_id}")

        except Exception as e:
            self.logger.error(f"Failed to process photo {photo_data.get('id', 'unknown')}: {str(e)}")
            raise

    def crawl_photos(self) -> List[Dict[str, Any]]:
        """Crawl all photos"""
        photos = []
        page = 0

        while True:
            page += 1
            posts = self.get_posts(page=page)

            if not posts:
                break

            photos.extend(posts)
            self.logger.info(f"Retrieved page {page} with {len(posts)} posts")

        return photos

def main():
    # Load environment variables
    load_dotenv()

    # Initialize client
    try:
        client = TransparentClassroomClient(
            email=os.getenv('TC_EMAIL'),
            password=os.getenv('TC_PASSWORD'),
            school_id=int(os.getenv('SCHOOL', 0)),
            child_id=int(os.getenv('CHILD', 0)),
            school_lat=float(os.getenv('SCHOOL_LAT', 0)),
            school_lng=float(os.getenv('SCHOOL_LNG', 0)),
            school_keywords=os.getenv('SCHOOL_KEYWORDS', '')
        )

        # Crawl photos
        photos = client.crawl_photos()

        # Process each photo
        for photo_data in photos:
            client.download_and_embed_metadata(photo_data)

        print(f"Successfully processed {len(photos)} photos")

    except Exception as e:
        logging.error(f"Application failed: {str(e)}")
        raise

if __name__ == "__main__":
    main()
